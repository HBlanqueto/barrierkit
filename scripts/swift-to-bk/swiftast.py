import re
import subprocess

TOKEN = re.compile(r'\s*(?:(\()|(\))|("(?:[^"\\]|\\.)*")|(\[[^\]]*\])|(<[^>]*>)|([^\s()"\[<]+))')


class Node:
    def __init__(self, kind, label=None):
        self.kind = kind
        self.label = label
        self.attrs = {}
        self.names = []
        self.children = []

    def get(self, key, default=None):
        return self.attrs.get(key, default)

    def walk(self):
        yield self
        for child in self.children:
            yield from child.walk()

    def child(self, kind):
        return next((c for c in self.children if c.kind == kind), None)


def unquote(text):
    return text[1:-1].replace('\\"', '"').replace('\\\\', '\\').replace('\\n', '\n')


def parse(text):
    stack = []
    root = None
    key = None
    for m in TOKEN.finditer(text):
        open_, close, quoted, bracket, angle, word = m.groups()
        if open_:
            stack.append(None)
        elif close:
            node = stack.pop()
            if stack:
                stack[-1].children.append(node)
            else:
                root = node
        elif stack[-1] is None:
            label, _, kind = (word or angle or quoted).rpartition('=')
            stack[-1] = Node(kind, label or None)
        else:
            node = stack[-1]
            if key is not None:
                node.attrs[key] = unquote(quoted) if quoted else (bracket or angle or word)
                key = None
            elif word and word.endswith('='):
                key = word[:-1]
            elif quoted:
                node.names.append(unquote(quoted))
            elif word and '=' in word:
                name, _, value = word.partition('=')
                node.attrs[name] = value
            elif word:
                node.attrs[word] = True
    return root


def dump_ast(path):
    result = subprocess.run(
        ['swiftc', '-typecheck', path, '-Xfrontend', '-dump-ast'],
        capture_output=True, text=True, check=False)
    return result.stdout, result.stderr
