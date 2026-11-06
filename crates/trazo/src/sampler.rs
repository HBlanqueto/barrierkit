use std::collections::VecDeque;
use std::fs;

pub const HISTORY: usize = 60;

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct CpuTimes {
    pub user: u64,
    pub system: u64,
    pub total: u64,
}

impl CpuTimes {
    pub fn share_since(&self, before: &Self) -> (f64, f64) {
        let total = self.total.saturating_sub(before.total).max(1) as f64;
        (
            self.user.saturating_sub(before.user) as f64 / total * 100.0,
            self.system.saturating_sub(before.system) as f64 / total * 100.0,
        )
    }
}

pub fn parse_cpu(line: &str) -> Option<CpuTimes> {
    let fields: Vec<u64> = line
        .split_whitespace()
        .skip(1)
        .filter_map(|field| field.parse().ok())
        .collect();
    let [user, nice, system, ..] = fields[..] else {
        return None;
    };
    Some(CpuTimes {
        user: user + nice,
        system,
        total: fields.iter().take(8).sum(),
    })
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Memory {
    pub used: f64,
    pub cached: f64,
    pub free: f64,
}

pub fn parse_meminfo(text: &str) -> Memory {
    let kib = |key: &str| -> f64 {
        text.lines()
            .find_map(|line| {
                line.strip_prefix(key)?
                    .split_whitespace()
                    .next()?
                    .parse()
                    .ok()
            })
            .unwrap_or(0.0)
    };
    let (total, free, cached) = (
        kib("MemTotal:"),
        kib("MemFree:"),
        kib("Cached:") + kib("Buffers:"),
    );
    let mib = |value: f64| value / 1024.0;
    Memory {
        used: mib((total - free - cached).max(0.0)),
        cached: mib(cached),
        free: mib(free),
    }
}

pub fn parse_net(text: &str) -> (u64, u64) {
    text.lines()
        .skip(2)
        .filter_map(|line| {
            let (name, data) = line.split_once(':')?;
            if name.trim() == "lo" {
                return None;
            }
            let fields: Vec<u64> = data
                .split_whitespace()
                .filter_map(|f| f.parse().ok())
                .collect();
            Some((*fields.first()?, *fields.get(8)?))
        })
        .fold((0, 0), |(rx, tx), (r, t)| (rx + r, tx + t))
}

#[derive(Clone, Debug, PartialEq)]
pub struct Process {
    pub name: String,
    pub pid: u32,
    pub memory: f64,
    pub threads: u32,
}

fn processes() -> Vec<Process> {
    let Ok(entries) = fs::read_dir("/proc") else {
        return Vec::new();
    };
    let mut list: Vec<Process> = entries
        .filter_map(|entry| {
            let pid: u32 = entry.ok()?.file_name().to_str()?.parse().ok()?;
            let status = fs::read_to_string(format!("/proc/{pid}/status")).ok()?;
            let field = |key: &str| {
                status
                    .lines()
                    .find_map(|l| l.strip_prefix(key).map(str::trim))
            };
            let rss: f64 = field("VmRSS:")?.split_whitespace().next()?.parse().ok()?;
            Some(Process {
                name: field("Name:")?.to_string(),
                pid,
                memory: rss / 1024.0,
                threads: field("Threads:")?.parse().ok()?,
            })
        })
        .collect();
    list.sort_by(|a, b| b.memory.total_cmp(&a.memory));
    list.truncate(12);
    list
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Monitor {
    pub user: VecDeque<f64>,
    pub system: VecDeque<f64>,
    pub cores: Vec<VecDeque<f64>>,
    pub received: VecDeque<f64>,
    pub sent: VecDeque<f64>,
    pub memory: Memory,
    pub load: [f64; 3],
    pub processes: Vec<Process>,
    last_cpu: Vec<CpuTimes>,
    last_net: Option<(u64, u64)>,
}

fn push(history: &mut VecDeque<f64>, value: f64) {
    if history.len() == HISTORY {
        history.pop_front();
    }
    history.push_back(value);
}

impl Monitor {
    pub fn sample(&mut self, seconds: f64) {
        let stat = fs::read_to_string("/proc/stat").unwrap_or_default();
        let cpus: Vec<CpuTimes> = stat
            .lines()
            .take_while(|line| line.starts_with("cpu"))
            .filter_map(parse_cpu)
            .collect();
        if self.last_cpu.len() == cpus.len() && !cpus.is_empty() {
            let (user, system) = cpus[0].share_since(&self.last_cpu[0]);
            push(&mut self.user, user);
            push(&mut self.system, system);
            self.cores.resize_with(cpus.len() - 1, VecDeque::new);
            for (core, (now, before)) in self
                .cores
                .iter_mut()
                .zip(cpus[1..].iter().zip(&self.last_cpu[1..]))
            {
                let (user, system) = now.share_since(before);
                push(core, user + system);
            }
        }
        self.last_cpu = cpus;

        let net = parse_net(&fs::read_to_string("/proc/net/dev").unwrap_or_default());
        if let Some((rx, tx)) = self.last_net {
            let rate = |now: u64, before: u64| now.saturating_sub(before) as f64 / 1024.0 / seconds;
            push(&mut self.received, rate(net.0, rx));
            push(&mut self.sent, rate(net.1, tx));
        }
        self.last_net = Some(net);

        self.memory = parse_meminfo(&fs::read_to_string("/proc/meminfo").unwrap_or_default());
        let load = fs::read_to_string("/proc/loadavg").unwrap_or_default();
        for (slot, value) in self.load.iter_mut().zip(load.split_whitespace()) {
            *slot = value.parse().unwrap_or(0.0);
        }
        self.processes = processes();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cpu_share_counts_nice_as_user_time() {
        let before = parse_cpu("cpu  100 0 50 850 0 0 0 0 0 0").unwrap();
        let after = parse_cpu("cpu  150 10 70 870 0 0 0 0 0 0").unwrap();
        let (user, system) = after.share_since(&before);
        assert_eq!((user.round(), system.round()), (60.0, 20.0));
    }

    #[test]
    fn meminfo_splits_used_cached_and_free_in_mib() {
        let text = "MemTotal: 4096 kB\nMemFree: 1024 kB\nBuffers: 512 kB\nCached: 512 kB\n";
        let memory = parse_meminfo(text);
        assert_eq!((memory.used, memory.cached, memory.free), (2.0, 1.0, 1.0));
    }

    #[test]
    fn network_totals_skip_the_loopback_interface() {
        let text = "Inter-|\n face |\n    lo: 999 0 0 0 0 0 0 0 999 0 0 0 0 0 0 0\n  eth0: 100 0 0 0 0 0 0 0 40 0 0 0 0 0 0 0\n";
        assert_eq!(parse_net(text), (100, 40));
    }
}
