"""Calculator — evaluate arithmetic expressions."""


def tokenize(expr: str) -> list:
    """Tokenize an expression into numbers, operators, and parentheses."""
    tokens = []
    i = 0
    while i < len(expr):
        c = expr[i]
        if c.isspace():
            i += 1
            continue
        if c in "+-*/()":
            tokens.append(c)
            i += 1
        elif c.isdigit() or c == ".":
            start = i
            while i < len(expr) and (expr[i].isdigit() or expr[i] == "."):
                i += 1
            tokens.append(float(expr[start:i]))
        else:
            raise ValueError(f"unexpected character: {c}")
    return tokens


class Parser:
    """Recursive descent parser for arithmetic expressions.

    Grammar:
        expr   -> term (('+' | '-') term)*
        term   -> unary (('*' | '/') unary)*
        unary  -> '-' unary | atom
        atom   -> NUMBER | '(' expr ')'
    """

    def __init__(self, tokens: list):
        self.tokens = tokens
        self.pos = 0

    def peek(self):
        if self.pos < len(self.tokens):
            return self.tokens[self.pos]
        return None

    def consume(self):
        tok = self.tokens[self.pos]
        self.pos += 1
        return tok

    def expect(self, expected):
        tok = self.peek()
        if tok != expected:
            raise ValueError(f"expected '{expected}', got '{tok}'")
        return self.consume()

    def parse_expr(self) -> float:
        left = self.parse_term()
        while self.peek() in ("+", "-"):
            op = self.consume()
            right = self.parse_term()
            if op == "+":
                left += right
            else:
                left -= right
        return left

    def parse_term(self) -> float:
        left = self.parse_unary()
        while self.peek() in ("*", "/"):
            op = self.consume()
            right = self.parse_unary()
            if op == "*":
                left *= right
            else:
                if right == 0:
                    raise ValueError("division by zero")
                left /= right
        return left

    def parse_unary(self) -> float:
        if self.peek() == "-":
            self.consume()
            return -self.parse_unary()
        if self.peek() == "+":
            self.consume()
            return self.parse_unary()
        return self.parse_atom()

    def parse_atom(self) -> float:
        tok = self.peek()
        if tok == "(":
            self.consume()
            result = self.parse_expr()
            self.expect(")")
            return result
        if isinstance(tok, float):
            self.consume()
            return tok
        raise ValueError(f"unexpected token: {tok}")


def evaluate(expr_string: str) -> float:
    """Evaluate an arithmetic expression string and return the result."""
    tokens = tokenize(expr_string)
    if not tokens:
        raise ValueError("empty expression")
    parser = Parser(tokens)
    result = parser.parse_expr()
    if parser.pos < len(parser.tokens):
        raise ValueError(f"unexpected token after expression: {parser.peek()}")
    return result


if __name__ == "__main__":
    examples = [
        "2 + 3 * 4",
        "(2 + 3) * 4",
        "10 / 3",
        "-(5 + 3)",
        "3.14 * 2",
        "((1 + 2) * (3 + 4)) / 7",
    ]
    for expr in examples:
        print(f"{expr} = {evaluate(expr)}")
