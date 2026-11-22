import json
import re
import subprocess
import sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE))
import swiftast  # noqa: E402

TABLE = {row['swift']: row for row in json.loads((HERE / 'table.json').read_text())}
SWIFTUI = ('SwiftUI', 'SwiftUICore')
COPY_TYPES = ('Int', 'Double', 'Float', 'CGFloat', 'Bool')
RUST_TYPES = {'Int': 'i64', 'Double': 'f64', 'Float': 'f32', 'CGFloat': 'f32', 'Bool': 'bool', 'String': 'String'}
FONTS = {'largeTitle': 'LARGE_TITLE', 'title': 'TITLE', 'title2': 'TITLE_2', 'title3': 'TITLE_3',
         'headline': 'HEADLINE', 'subheadline': 'SUBHEADLINE', 'body': 'BODY', 'callout': 'CALLOUT',
         'footnote': 'FOOTNOTE', 'caption': 'CAPTION', 'caption2': 'CAPTION_2'}
BUTTON_STYLES = {'automatic': None, 'bordered': None, 'borderedProminent': 'bordered_prominent',
                 'borderless': 'borderless', 'plain': 'plain', 'glass': 'glass', 'glassProminent': 'glass_prominent'}
ALIGNMENTS = {'leading': 'Start', 'trailing': 'End', 'center': 'Center', 'top': 'Start', 'bottom': 'End'}


class Unsupported(Exception):
    def __init__(self, key, expr=False):
        super().__init__(key)
        self.key, self.expr = key, expr


class Decl:
    def __init__(self, text):
        text = re.sub(r' \[with.*$', '', text)
        self.module, _, text = text.partition('.(file).')
        text = text.split('@')[0]
        m = re.match(r'^(.*?)\((.*)\)$', text)
        self.labels = [x or '_' for x in m.group(2).split(':')[:-1]] if m else None
        text = m.group(1) if m else text
        text = re.sub(r' extension(?=\.)', '', text)
        for op in ('..<', '...'):
            if text.endswith(op):
                owner, member = text[:-len(op)].rstrip('.'), op
                break
        else:
            owner, _, member = text.rpartition('.')
        self.owner, self.member = owner, member


class Call:
    def __init__(self, decl, base, args):
        self.decl, self.base, self.args = decl, base, args
        self.owner, self.member, self.module = decl.owner, decl.member, decl.module

    def labels_text(self):
        return ''.join(f'{name}:' for name, _ in self.args)

    def arg(self, label, position=None):
        for i, (name, node) in enumerate(self.args):
            if name == label or (position is not None and name == '_' and i == position):
                return node
        return None


def expr_children(node):
    return [c for c in node.children if c.kind != 'default_argument_expr']


def arguments(arg_list):
    out = []
    for arg in arg_list.children if arg_list else []:
        inner = expr_children(arg)
        if arg.kind == 'argument' and inner:
            out.append((arg.get('label', '_'), inner[0]))
    return out


def declref(node):
    return Decl(node.attrs['decl'])


def as_call(node):
    kids = node.children
    if node.kind == 'call_expr' and kids and kids[0].kind == 'constructor_ref_call_expr':
        return Call(declref(kids[0].children[0]), None, arguments(kids[1] if len(kids) > 1 else None))
    if node.kind == 'call_expr' and kids and kids[0].kind == 'dot_syntax_call_expr':
        head = kids[0]
        base = arguments(head.children[1])[0][1] if len(head.children) > 1 else None
        return Call(declref(head.children[0]), base, arguments(kids[1] if len(kids) > 1 else None))
    if node.kind in ('binary_expr', 'prefix_unary_expr') and len(kids) > 1:
        return Call(declref(kids[0].children[0]), None, arguments(kids[1]))
    raise Unsupported(node.kind)


def single(node):
    kids = expr_children(node)
    return kids[0] if kids else node


def ident(name):
    return re.sub(r'(?<=[a-z0-9])(?=[A-Z])', '_', name).lower()


def strip(node):
    while node.kind in ('inject_into_optional', 'load_expr', 'paren_expr', 'argument', 'underlying_to_opaque_expr'):
        node = single(node)
    return node


def rust_str(text):
    return json.dumps(text, ensure_ascii=False)


def lookup(key):
    for candidate in (key, 'View.' + key.split('.', 1)[-1] if '.' in key else key, key.replace('.init', '')):
        if candidate in TABLE:
            return TABLE[candidate]
    return None


def gap_ref(key, expr=False):
    row = lookup(key)
    if row is None:
        return 'translator: Swift expression not lowered' if expr else 'no row in swiftui-api-gap.md or the glossary'
    kit = f" (kit: {row['kit']})" if row.get('kit') else ''
    if row['status'] == 'n/a':
        return f"n/a: {row.get('gap', key)}"
    if row['status'] == 'missing' or (row.get('gap') and row['origin'] == 'gap' and row['status'] != 'mapped'):
        return row['gap'] + kit
    return f'translator: not lowered yet{kit}' + (f" ({row['gap']})" if row.get('gap') else '')


class Struct:
    def __init__(self, node):
        self.node = node
        self.name = node.names[0]
        self.inherits = set(re.split(r'[,\s]+', node.get('inherits', '')))
        members = node.children
        names = {n: v for v in members if v.kind == 'var_decl' for n in v.names[:1]}
        inits = {}
        for pbd in members:
            if pbd.kind != 'pattern_binding_decl':
                continue
            for entry in pbd.children:
                named = next((p for p in entry.walk() if p.kind == 'pattern_named'), None)
                init = next((x for x in entry.children if x.label == 'original_init'), None)
                if named and init is not None:
                    inits[named.names[0]] = init
        self.states, self.props, self.bindings = {}, {}, {}
        for name, var in names.items():
            if name.startswith(('$', '_')) or name == 'body':
                continue
            type_ = var.get('interface_type', '')
            if '__' + name in names and names['__' + name].get('interface_type', '').startswith('LazyState<'):
                self.states[name] = (type_, inits.get(name))
            elif '_' + name in names and names['_' + name].get('interface_type', '').startswith('Binding<'):
                self.bindings[name] = type_
            elif var.get('readImpl') == 'stored':
                self.props[name] = (type_, inits.get(name))
        self.body = next((v for v in members if v.kind == 'var_decl' and v.names[:1] == ['body']), None)
        self.mutated = set()

    def body_brace(self):
        accessor = self.body.child('accessor_decl')
        return accessor.child('brace_stmt')


class Context:
    def __init__(self, structs):
        self.structs = structs
        self.struct = None
        self.report = []
        self.locals = set()
        self.defs = []
        self.imports = set()
        self.prop_consts = {}

    def todo(self, key, expr=False):
        line = f"TODO(swift: {key}) -> {gap_ref(key, expr)}"
        self.report.append(line)
        return '// ' + line


class View:
    def __init__(self, base, control=False):
        self.base = base
        self.control = control
        self.native, self.events, self.outer, self.last = [], [], [], []
        self.segs = []
        self.todos = []
        self.render = None

    def code(self):
        parts = [self.base, *self.native, *self.events]
        if not self.control:
            parts += self.outer
        body = ''.join(parts)
        for kind, text in self.segs:
            body += f'\n{text}\n' if kind == 'todo' else f'.{kind}({text})'
        if self.control and self.outer:
            body = 'rect()' + ''.join(self.outer) + f'.child({body})'
        return body + ''.join(self.last)


def member_chain(node):
    return Decl(node.attrs['decl']), (expr_children(node)[0] if expr_children(node) else None)


def state_of(node, c):
    node = single(node) if node.kind in ('load_expr', 'inout_expr', 'argument', 'paren_expr') else node
    if node.kind != 'member_ref_expr':
        return None
    decl, base = member_chain(node)
    name = decl.member.lstrip('$')
    if decl.owner == c.struct.name and name in c.struct.states:
        return name
    return None


def read_state(name, c, display=False):
    type_ = c.struct.states[name][0]
    name = ident(name)
    if display or type_ not in COPY_TYPES:
        return f'{name}.read().clone()' if type_ == 'String' and not display else f'{name}.read()'
    return f'*{name}.read()'


def rx(node, c, want='value'):
    kind = node.kind
    if kind in ('load_expr', 'paren_expr', 'inject_into_optional', 'argument', 'force_value_expr',
                'inout_expr', 'dot_syntax_base_ignored'):
        return rx(single(node), c, want)
    if kind == 'integer_literal_expr':
        return node.get('value') + ('.0' if want == 'f32' else '')
    if kind == 'float_literal_expr':
        return node.get('value')
    if kind == 'boolean_literal_expr':
        return node.get('value')
    if kind == 'string_literal_expr':
        return rust_str(node.get('value'))
    if kind == 'interpolated_string_literal_expr':
        return interpolation(node, c)
    if kind == 'ternary_expr':
        cond, a, b = (rx(x, c) for x in expr_children(node)[:3])
        return f'if {cond} {{ {a} }} else {{ {b} }}'
    if kind == 'autoclosure_expr':
        return rx(expr_children(node)[-1], c, want)
    if kind == 'declref_expr':
        name = declref(node).member
        if name in c.locals:
            return ident(name)
        raise Unsupported(declref(node).owner + '.' + name, True)
    if kind == 'member_ref_expr':
        return member_ref(node, c, want)
    if kind in ('binary_expr', 'prefix_unary_expr'):
        return operator(node, c, want)
    raise Unsupported(kind, True)


def member_ref(node, c, want):
    decl, base = member_chain(node)
    if base is not None and base.kind == 'declref_expr' and declref(base).member == 'self':
        name = decl.member
        if name in c.struct.states:
            return read_state(name, c, want == 'display')
        if name in c.struct.bindings:
            type_ = c.struct.bindings[name]
            return f'*self.{ident(name)}.read()' if type_ in COPY_TYPES else f'self.{ident(name)}.read().clone()'
        if name in c.struct.props:
            type_, init = c.struct.props[name]
            if init is not None and init.kind == 'array_expr':
                return array_const(name, init, c)
            return f'self.{ident(name)}' + ('.clone()' if type_ == 'String' and want != 'display' else '')
        raise Unsupported(f'{decl.owner}.{name}', True)
    if base is not None and base.kind == 'declref_expr' and declref(base).member in c.locals:
        return f'{ident(declref(base).member)}.{ident(decl.member)}'
    if base is not None and base.kind == 'type_expr':
        raise Unsupported(f'{decl.owner}.{decl.member}', True)
    raise Unsupported(f'{decl.owner}.{decl.member}', True)


STATIC_TYPES = {'Int': 'i64', 'Double': 'f64', 'Bool': 'bool', 'String': "&'static str"}


def struct_def(struct):
    fields = ''.join(f"    {ident(n)}: {STATIC_TYPES.get(t, 'String')},\n" for n, (t, _) in struct.props.items())
    return f'struct {struct.name} {{\n{fields}}}\n'


def array_const(name, node, c):
    key = (c.struct.name, name)
    if key not in c.prop_consts:
        rust = re.sub(r'(?<=[a-z0-9])(?=[A-Z])', '_', name).upper()
        items = []
        for elem in expr_children(node):
            call = try_call(elem) if elem.kind == 'call_expr' else None
            if call and call.owner in c.structs:
                definition = struct_def(c.structs[call.owner])
                if definition not in c.defs:
                    c.defs.append(definition)
                items.append(f"{call.owner} {{ {', '.join(f'{ident(n)}: {rx(v, c)}' for n, v in call.args)} }}")
                element = call.owner
            else:
                items.append(rx(elem, c))
                element = STATIC_TYPES.get(re.sub(r'[\[\]]', '', node.get('type', '')), 'i64')
        c.defs.append(f"const {rust}: [{element}; {len(items)}] = [{', '.join(items)}];\n")
        c.prop_consts[key] = rust
    return c.prop_consts[key]


PRECEDENCE = {'||': 1, '&&': 2, '==': 3, '!=': 3, '<': 3, '>': 3, '<=': 3, '>=': 3,
              '+': 4, '-': 4, '*': 5, '/': 5, '%': 5}


def operator(node, c, want):
    call = as_call(node)
    op = call.member
    args = [a for _, a in call.args]
    if node.kind == 'prefix_unary_expr' and op == '!':
        return '!' + paren(args[0], c, 9)
    if op in ('..<', '...') and len(args) == 2:
        return f"({rx(args[0], c)}..{'=' if op == '...' else ''}{rx(args[1], c)})"
    if op in PRECEDENCE and len(args) == 2 and call.owner != 'String':
        return f'{paren(args[0], c, PRECEDENCE[op])} {op} {paren(args[1], c, PRECEDENCE[op] + 1)}'
    raise Unsupported(f'{call.owner}.{op}', True)


def paren(node, c, level):
    text = rx(node, c)
    inner = strip(node)
    if inner.kind == 'ternary_expr':
        return f'({text})'
    if inner.kind == 'binary_expr' and PRECEDENCE.get(as_call(inner).member, 9) < level:
        return f'({text})'
    return text


def interpolation(node, c):
    tap = node.child('tap_expr')
    pieces = []
    for call in (tap.child('brace_stmt').children if tap else []):
        if call.kind != 'call_expr':
            continue
        parsed = as_call(call)
        if parsed.member == 'appendLiteral':
            pieces.append(('lit', parsed.args[0][1].get('value')))
        elif parsed.member == 'appendInterpolation' and len(parsed.args) == 1:
            pieces.append(('expr', rx(parsed.args[0][1], c, 'display')))
        else:
            raise Unsupported(f'LocalizedStringKey.StringInterpolation.{parsed.member}', True)
    exprs = [p for p in pieces if p[0] == 'expr']
    if not exprs:
        return rust_str(''.join(p[1] for p in pieces))
    if len(exprs) == 1 and all(p[0] == 'expr' or p[1] == '' for p in pieces):
        return f'{exprs[0][1]}.to_string()'
    inline = lambda p: re.fullmatch(r'[A-Za-z_]\w*', p[1]) is not None
    text = ''.join(p[1].replace('{', '{{').replace('}', '}}') if p[0] == 'lit' else '{' + (p[1] if inline(p) else '') + '}'
                   for p in pieces)
    args = ''.join(f', {p[1]}' for p in exprs if not inline(p))
    return f'format!({rust_str(text)}{args})'


def statements(closure, c):
    brace = closure.child('brace_stmt')
    flat = [single(s) if s.kind == 'return_stmt' else s for s in (brace.children if brace else [])]
    return [s for s in flat if s.kind in ('binary_expr', 'assign_expr', 'call_expr')]


def statement(node, c):
    if node.kind == 'assign_expr':
        lhs, rhs = expr_children(node)[:2]
        name = state_of(lhs, c)
        if name is None:
            raise Unsupported('assignment', True)
        c.struct.mutated.add(name)
        return f'{ident(name)}.set({rx(rhs, c)})'
    call = as_call(node)
    if node.kind == 'binary_expr' and call.member in ('+=', '-=', '*=', '/='):
        name = state_of(call.args[0][1], c)
        if name is None:
            raise Unsupported(f'{call.owner}.{call.member}', True)
        c.struct.mutated.add(name)
        return f'*{ident(name)}.write() {call.member} {rx(call.args[1][1], c)}'
    if call.owner == 'Bool' and call.member == 'toggle':
        name = state_of(call.base, c)
        if name is None:
            raise Unsupported('Bool.toggle', True)
        c.struct.mutated.add(name)
        return f'{ident(name)}.toggle()'
    raise Unsupported(f'{call.owner}.{call.member}', True)


def action(closure, c, params='_'):
    stmts = []
    for s in statements(closure, c):
        try:
            stmts.append(statement(s, c))
        except Unsupported as e:
            stmts.append(c.todo(e.key, e.expr))
    if len(stmts) == 1 and not stmts[0].startswith('//'):
        return f'move |{params}| {stmts[0]}'
    body = ''.join(s + ('\n' if s.startswith('//') else ';\n') for s in stmts)
    return f'move |{params}| {{\n{body}}}'


def binding(node, c):
    node = single(node) if node.kind == 'argument' else node
    if node.kind == 'member_ref_expr':
        decl, base = member_chain(node)
        name = decl.member
        if name.startswith('$') and decl.owner == c.struct.name and name[1:] in c.struct.states:
            return name[1:], False
        if name.startswith('$') and name[1:] in c.struct.bindings:
            return name[1:], True
    raise Unsupported('Binding')


def binding_ref(name, is_prop):
    return f'self.{ident(name)}' if is_prop else ident(name)


def builder_items(brace):
    items = []
    for st in brace.children if brace else []:
        if st.kind == 'pattern_binding_decl':
            for entry in st.children:
                init = next((x for x in entry.children if x.label == 'original_init'), None)
                if init is None:
                    continue
                if init.kind == 'call_expr' and init.children[0].kind == 'dot_syntax_call_expr' \
                        and Decl(init.children[0].children[0].attrs.get('decl', 'x.(file).x.x')).owner.startswith('ViewBuilder'):
                    continue
                items.append(('view', init))
        elif st.kind == 'if_stmt':
            items.append(parse_if(st))
        elif st.kind in ('switch_stmt', 'for_each_stmt'):
            items.append(('stmt', st.kind))
    return items


def parse_if(node):
    conds = next(k for k in node.children if k.label == 'conditions').children
    arms = [k for k in node.children if k.kind in ('brace_stmt', 'if_stmt')]
    other = None
    if len(arms) > 1:
        other = [parse_if(arms[1])] if arms[1].kind == 'if_stmt' else builder_items(arms[1]) or None
    return ('if', conds, builder_items(arms[0]), other)


def views_of(items, c):
    segs = []
    for item in items:
        if item[0] == 'view':
            segs += child_segments(item[1], c)
        elif item[0] == 'if':
            segs += conditional(item, c)
        else:
            segs.append(('todo', c.todo(item[1])))
    return segs


def child_segments(node, c):
    call = try_call(node)
    if call is not None and call.base is None and call.owner == 'ForEach' and call.module in SWIFTUI:
        return for_each(call, c)
    v = view(node, c)
    return [('todo', t) for t in v.todos] + [('child', v.code())]


def try_call(node):
    try:
        return as_call(node)
    except Unsupported:
        return None


def cond_call(cond):
    return cond if re.fullmatch(r'[\w.]+(\(\))?', cond) else f'({cond})'


def conditional(item, c):
    _, conds, then, other = item
    try:
        cond = ' && '.join(rx(x, c) for x in conds)
    except Unsupported as e:
        return [('todo', c.todo(e.key, e.expr))]
    then_segs = views_of(then, c)
    other_segs = views_of(other, c) if other is not None else []
    todos = [s for s in then_segs + other_segs if s[0] == 'todo']
    then_views = [s for s in then_segs if s[0] != 'todo']
    other_views = [s for s in other_segs if s[0] != 'todo']
    if any(kind == 'children' for kind, _ in then_views + other_views):
        return [*todos, ('todo', c.todo('ForEach (inside if)'))]
    if other is None:
        return [*todos, *(('maybe_child', f'{cond_call(cond)}.then(|| {t})') for _, t in then_views)]
    if len(then_views) == len(other_views) == 1:
        a, b = then_views[0][1], other_views[0][1]
        return [*todos, ('child', f'if {cond} {{ {a}.into_element() }} else {{ {b}.into_element() }}')]
    arm = lambda views: ', '.join(t + '.into_element()' for _, t in views)
    return [*todos, ('children', f'if {cond} {{ vec![{arm(then_views)}] }} else {{ vec![{arm(other_views)}] }}')]


def for_each_parts(data, closure, c, call=None):
    param = closure.child('parameter_list').child('parameter').names[0]
    c.locals.add(param)
    try:
        source = rx(data, c)
        items = builder_items(closure.child('brace_stmt'))
        if len(items) != 1 or items[0][0] != 'view':
            raise Unsupported('ForEach.multiple-views')
        return source, param, view(items[0][1], c)
    finally:
        c.locals.discard(param)


def for_each(call, c):
    data = call.args[0][1]
    try:
        source, param, v = for_each_parts(data, call.arg('content'), c)
    except Unsupported as e:
        return [('todo', c.todo(e.key, e.expr))]
    v.last.append(f".key({id_key(call, strip(data), param)})")
    mapper = '' if source.startswith('(') else '.into_iter()'
    return [*(('todo', t) for t in v.todos), ('children', f'{source}{mapper}.map(|{ident(param)}| {v.code()})')]


def list_view(call, c):
    closures = [n for _, n in call.args if n.kind == 'closure_expr']
    plain = [n for name, n in call.args if n.kind != 'closure_expr' and name == '_']
    if plain and closures:
        data, closure = plain[0], closures[0]
    else:
        inner = builder_items(closures[0].child('brace_stmt')) if closures else []
        each = try_call(inner[0][1]) if len(inner) == 1 and inner[0][0] == 'view' else None
        if each is None or each.owner != 'ForEach':
            raise Unsupported('List.init(static rows)')
        data, closure = each.args[0][1], each.arg('content')
    source, param, v = for_each_parts(data, closure, c)
    if source.startswith('('):
        raise Unsupported('List.init(range)')
    first = re.search(r'Text::new\(("[^"]*"|[\w.]+)\)', v.code())
    name = f'{first.group(1)}.to_string()' if first else 'String::new()'
    c.imports.add('use bk::views::list::List;')
    param = ident(param)
    rows = f'{source}.iter().map(|{param}| {name}).collect()'
    out = View(f'List::new("List", {rows})', control=True)
    out.native.append(f'.row(|index| {{ let {param} = &{source}[index]; {v.code()} }})')
    return out


def id_key(call, data, param):
    id_arg = call.arg('id')
    if id_arg is not None:
        member = next((n for n in id_arg.walk() if n.kind == 'member'), None)
        if member is None or 'decl' not in member.attrs:
            return ident(param)
        return f'{ident(param)}.{ident(Decl(member.attrs["decl"]).member)}'
    if data.get('type', '').startswith(('Range<', 'ClosedRange<')):
        return ident(param)
    return f'{ident(param)}.id'


def view(node, c):
    try:
        call = as_call(node)
    except Unsupported as e:
        return placeholder(c, e.key)
    if call.base is None:
        return constructor(call, c)
    v = view(call.base, c)
    handler = MODIFIERS.get(call.member) if call.module in SWIFTUI else None
    try:
        if handler is None:
            raise Unsupported(f'View.{call.member}')
        handler(v, call, c)
    except Unsupported as e:
        v.todos.append(c.todo(e.key, e.expr))
    return v


def placeholder(c, key):
    v = View('rect()')
    v.todos.append(c.todo(key))
    return v


def constructor(call, c):
    if call.module in SWIFTUI:
        handler = CONSTRUCTORS.get(call.owner)
        if handler is None:
            v = placeholder(c, call.owner + '.init')
            content = call.arg('content') or next((n for _, n in call.args if n.kind == 'closure_expr'), None)
            if content is not None:
                v.segs = views_of(builder_items(content.child('brace_stmt')), c)
            return v
        try:
            return handler(call, c)
        except Unsupported as e:
            return placeholder(c, e.key)
    if call.owner in c.structs and 'View' in c.structs[call.owner].inherits:
        return component(call, c)
    return placeholder(c, call.owner + '.init')


def component(call, c):
    target = c.structs[call.owner]
    fields = []
    for name, node in call.args:
        if name in target.bindings:
            state, is_prop = binding(node, c)
            ref = binding_ref(state, is_prop)
            fields.append(ident(name) if ref == ident(name) else f'{ident(name)}: {ref}')
        else:
            text = rx(node, c)
            type_ = target.props.get(name, ('',))[0]
            fields.append(f'{ident(name)}: {text}.to_string()' if type_ == 'String' and text.startswith('"') else f'{ident(name)}: {text}')
    return View(f"{call.owner} {{ {', '.join(fields)} }}" if fields else call.owner)


def container(rust, call, c):
    v = View(rust + '::new()')
    alignment = call.arg('alignment')
    if alignment is not None:
        v.native.append(f'.cross_align(Alignment::{alignment_of(alignment)})')
    spacing = call.arg('spacing')
    if spacing is not None:
        v.native.append(f'.spacing({rx(spacing, c, "f32")})')
    content = call.arg('content')
    v.segs = views_of(builder_items(content.child('brace_stmt')), c) if content is not None else []
    return v


def alignment_of(node):
    inner = strip(node)
    if inner.kind == 'member_ref_expr' and Decl(inner.attrs['decl']).member in ALIGNMENTS:
        return ALIGNMENTS[Decl(inner.attrs['decl']).member]
    raise Unsupported('Alignment')


def text(call, c):
    first = call.args[0][1]
    return View(f'Text::new({rx(first, c)})')


def button(call, c):
    title = call.arg('_', 0)
    handler = call.arg('action')
    label = call.arg('label')
    if handler is None or handler.kind != 'closure_expr' or (title is None) == (label is None):
        raise Unsupported('Button.init')
    if label is None:
        v = View(f'Button::new({rx(title, c)})', control=True)
    else:
        items = builder_items(label.child('brace_stmt'))
        if len(items) != 1 or items[0][0] != 'view':
            raise Unsupported('Button.init(label:)')
        inner = view(items[0][1], c)
        code = inner.code()
        named = re.match(r'(?:Text|Label)::new\(("[^"]*"|[\w.]+)', code)
        v = View(f'Button::new({named.group(1) if named else rust_str("Button")})', control=True)
        v.todos += inner.todos
        if not code.startswith('Text::new('):
            v.native.append(f'.label(|| {code})')
    v.events.append(f'.on_press({action(handler, c)})')
    return v


def symbol(node, c):
    name = rx_str(node)
    row = TABLE.get('SF:' + str(name))
    if row is None:
        raise Unsupported(f'SF:{name}')
    return row['kit']


def image(call, c):
    if call.labels_text() != 'systemName:':
        raise Unsupported('Image.init')
    return View(f"Image::system({symbol(call.arg('systemName'), c)})", control=True)


def label(call, c):
    if call.labels_text() != '_:systemImage:':
        raise Unsupported('Label.init')
    return View(f"Label::new({rx(call.arg('_', 0), c)}, {symbol(call.arg('systemImage'), c)})", control=True)


def toggle(call, c):
    title = call.arg('_', 0)
    state, is_prop = binding(call.arg('isOn'), c)
    ref = binding_ref(state, is_prop)
    v = View(f'Checkbox::new({rx(title, c)})', control=True)
    v.render = ('checkbox', ref)
    c.struct.mutated.add(state)
    v.native.append(f'.checked(*{ref}.read())')
    v.events.append(f'.on_toggle(move |checked| {ref}.set(checked))')
    return v


def text_field(call, c):
    title = call.arg('_', 0)
    state, is_prop = binding(call.arg('text'), c)
    return View(f'TextField::new({binding_ref(state, is_prop)}, {rx(title, c)})', control=True)


def leaf(rust):
    def build(call, c):
        if call.args:
            raise Unsupported(call.owner + '.init')
        return View(rust)
    return build


def scroll_view(call, c):
    if call.arg('_', 0) is not None:
        raise Unsupported('ScrollView.init')
    return container('ScrollView', call, c)


def length(node, c):
    return rx(node, c, 'f32')


def padding(v, call, c):
    labels = [name for name, _ in call.args]
    if not call.args:
        v.outer.append('.padding(SPACE_4)')
    elif labels == ['_']:
        v.outer.append(f'.padding({length(call.args[0][1], c)})')
    elif labels == ['_', '_']:
        edges = Decl(strip(call.args[0][1]).attrs['decl']).member
        amount = length(call.args[1][1], c)
        shapes = {'horizontal': f'(0.0, {amount})', 'vertical': f'({amount}, 0.0)', 'all': amount,
                  'top': f'({amount}, 0.0, 0.0, 0.0)', 'leading': f'(0.0, 0.0, 0.0, {amount})',
                  'bottom': f'(0.0, 0.0, {amount}, 0.0)', 'trailing': f'(0.0, {amount}, 0.0, 0.0)'}
        if edges not in shapes:
            raise Unsupported('View.padding')
        v.outer.append(f'.padding({shapes[edges]})')
    else:
        raise Unsupported('View.padding')


def infinity(node):
    return node.kind == 'member_ref_expr' and Decl(node.attrs['decl']).member == 'infinity' \
        or any(infinity(k) for k in node.children)


def frame(v, call, c):
    for name, node in call.args:
        if name in ('width', 'height'):
            v.outer.append(f'.{name}(Size::px({length(node, c)}))')
        elif name in ('maxWidth', 'maxHeight') and infinity(node):
            v.outer.append(f".{name[3:].lower()}(Size::fill())")
        elif name == 'alignment':
            raise Unsupported('View.frame(alignment:)')
        else:
            raise Unsupported(f'View.frame({name}:)')


def font(v, call, c):
    inner = strip(call.args[0][1])
    name = Decl(inner.attrs['decl']).member if inner.kind == 'member_ref_expr' else None
    if name not in FONTS:
        raise Unsupported('Font.' + str(name))
    if not v.base.startswith('Text::new'):
        raise Unsupported('View.font(on a container)')
    v.native.append(f'.font({FONTS[name]})')


def button_style(v, call, c):
    inner = strip(call.args[0][1])
    name = Decl(inner.attrs['decl']).member if inner.kind == 'member_ref_expr' else None
    if name not in BUTTON_STYLES:
        raise Unsupported('PrimitiveButtonStyle.' + str(name))
    if not v.base.startswith('Button::new'):
        raise Unsupported('View.buttonStyle(on a container)')
    if BUTTON_STYLES[name]:
        v.native.append(f'.style(ButtonStyle::{BUTTON_STYLES[name]})')


def toggle_style(v, call, c):
    inner = strip(call.args[0][1])
    name = Decl(inner.attrs['decl']).member if inner.kind == 'member_ref_expr' else None
    if not v.base.startswith('Checkbox::new') or name not in ('switch', 'checkbox', 'automatic'):
        raise Unsupported('View.toggleStyle')
    if name == 'switch':
        ref = v.render[1]
        v.base = v.base.replace('Checkbox::new', 'Switch::new')
        v.native = [f'.on(*{ref}.read())']
        v.events = [f'.on_toggle(move |checked| {ref}.set(checked))']


def disabled(v, call, c):
    flag = rx(call.args[0][1], c)
    (v.native if v.control else v.last).append(f'.disabled({flag})')


CONSTRUCTORS = {
    'VStack': lambda call, c: container('VStack', call, c),
    'HStack': lambda call, c: container('HStack', call, c),
    'ScrollView': scroll_view,
    'Text': text,
    'Button': button,
    'Toggle': toggle,
    'TextField': text_field,
    'Spacer': leaf('Spacer::new()'),
    'Divider': leaf('Divider::new()'),
    'List': list_view,
    'Image': image,
    'Label': label,
}
MODIFIERS = {
    'padding': padding, 'frame': frame, 'font': font, 'buttonStyle': button_style,
    'toggleStyle': toggle_style, 'disabled': disabled,
}


def initial(name, type_, node, c):
    if node is None:
        raise Unsupported('@State without initial value', True)
    value = rx(node, c)
    if type_ == 'String':
        return 'String::new' if value == '""' else f'|| {value}.to_string()'
    return f'|| {value}'


def state_lines(struct, c):
    lines = []
    for name, (type_, init) in struct.states.items():
        mut = 'mut ' if name in struct.mutated else ''
        try:
            lines.append(f'let {mut}{ident(name)} = use_state({initial(name, type_, init, c)});')
        except Unsupported as e:
            lines.append(c.todo(e.key, e.expr))
    return lines


def render_body(struct, c):
    c.struct = struct
    segs = views_of(builder_items(struct.body_brace()), c)
    heads = [text for kind, text in segs if kind == 'todo']
    views = [(k, t) for k, t in segs if k != 'todo']
    if len(views) == 1 and views[0][0] == 'child':
        return heads, views[0][1]
    wrapper = View('VStack::new()')
    wrapper.segs = segs
    heads = [c.todo('ViewBuilder (several root views)')]
    return heads, wrapper.code()


def block(struct, c):
    heads, expr = render_body(struct, c)
    return [*state_lines(struct, c), *heads, expr]


def component_source(struct, c):
    lines = block(struct, c)
    fields = []
    for name, (type_, _) in struct.props.items():
        fields.append(f'    {ident(name)}: {RUST_TYPES.get(type_, "String")},')
    for name, type_ in struct.bindings.items():
        fields.append(f'    {ident(name)}: State<{RUST_TYPES.get(type_, "String")}>,')
    body = '\n        '.join(lines)
    return (f'#[derive(PartialEq)]\nstruct {struct.name} {{\n' + '\n'.join(fields) + '\n}\n\n'
            f'impl Component for {struct.name} {{\n    fn render(&self) -> impl IntoElement {{\n        {body}\n    }}\n}}\n')


def kebab(name):
    return re.sub(r'(?<=[a-z0-9])(?=[A-Z])', '-', name).lower()


def words(name):
    return re.sub(r'(?<=[a-z0-9])(?=[A-Z])', ' ', name)


def find_scene(structs):
    app = next((s for s in structs.values() if 'App' in s.inherits), None)
    if app is None:
        return None, None, None, None
    brace = app.body_brace()
    group = None
    for node in brace.walk():
        call = try_call(node) if node.kind == 'call_expr' else None
        if call and call.owner == 'WindowGroup' and call.member == 'init':
            group = call
            break
    if group is None:
        return app, None, None, None
    title = next((rx_str(n) for name, n in group.args if name == '_' and n.kind != 'closure_expr'), None)
    content = group.arg('content')
    root = None
    for item in builder_items(content.child('brace_stmt')) if content is not None else []:
        cons = try_call(item[1]) if item[0] == 'view' else None
        if cons and cons.owner in structs:
            root = cons.owner
    size = None
    for node in brace.walk():
        call = try_call(node) if node.kind == 'call_expr' else None
        if call and call.member == 'defaultSize':
            size = tuple(rx_num(n) for _, n in call.args)
    return app, title, root, size


def rx_str(node):
    node = next((n for n in node.walk() if n.kind == 'string_literal_expr'), None)
    return node.get('value') if node is not None else None


def rx_num(node):
    inner = next((n for n in node.walk() if n.kind == 'integer_literal_expr'), None)
    return inner.get('value') if inner is not None else '0'


def translate(path, name=None, title=None, root=None):
    out, err = swiftast.dump_ast(path)
    if not out.startswith('(source_file'):
        raise SystemExit(err or 'swiftc produced no AST')
    tree = swiftast.parse(out)
    structs = {n.names[0]: Struct(n) for n in tree.children if n.kind == 'struct_decl'}
    c = Context(structs)
    views = [s for s in structs.values() if 'View' in s.inherits]
    app, scene_title, scene_root, size = find_scene(structs)
    used = {n.owner for s in views for n in (try_call(x) for x in s.body.walk() if x.kind == 'call_expr') if n}
    root_name = root or scene_root or next((s.name for s in views if s.name not in used), views[-1].name)
    root_struct = structs[root_name]
    others = [s for s in views if s is not root_struct]
    lines = block(root_struct, c)
    components = [component_source(s, c) for s in others]
    parts = ['\n'.join(sorted({'use bk::prelude::*;', *c.imports})) + '\n', *c.defs, *components]
    window_title = title or scene_title or words(root_name)
    app_name = name or kebab(root_name)
    options = f'\n.options(|window| window.size(({size[0]}, {size[1]})))' if size else ''
    body = '\n'.join(lines)
    parts.append(f'fn main() -> Result<(), RunError> {{\n    Application::new({rust_str(app_name)})\n'
                 f'        .window({rust_str(window_title)}, || {{\n{body}\n}}){options}\n        .run()\n}}\n')
    source = '\n'.join(parts)
    source = rustfmt(source)
    if c.report:
        counts = {}
        for line in c.report:
            counts[line] = counts.get(line, 0) + 1
        summary = ['', f'// Translation report: {len(c.report)} open']
        summary += [f'//   {line}' + (f' (x{n})' if n > 1 else '') for line, n in counts.items()]
        source += '\n'.join(summary) + '\n'
    return source, c.report


def rustfmt(source):
    result = subprocess.run(['rustfmt', '--edition', '2024'], input=source, capture_output=True, text=True)
    if result.returncode != 0:
        sys.stderr.write(result.stderr)
        return source
    return result.stdout


def main(argv):
    args = argv[1:]
    options = {}
    strict = '--strict' in args
    args = [a for a in args if a != '--strict']
    while args and args[0].startswith('--'):
        options[args[0][2:]] = args[1]
        args = args[2:]
    if len(args) != 1:
        sys.exit('usage: translate.py [--name N] [--title T] [--root R] [--strict] File.swift')
    source, report = translate(args[0], **options)
    sys.stdout.write(source)
    sys.stderr.write(f'swift-to-bk: {len(report)} open\n' + ''.join(f'  {line}\n' for line in report))
    return 1 if strict and report else 0


if __name__ == '__main__':
    sys.exit(main(sys.argv))
