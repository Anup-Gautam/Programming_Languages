use std::env;
use std::fs::File;
use std::io::prelude::*;
use sexp::*;
use sexp::Atom::*;

enum Expr{
    Num(i32),
    Add1(Box<Expr>),
    Sub1(Box<Expr>),
    Negate(Box<Expr>),
}

fn parse_expr(s: &Sexp) -> Expr {
    match s {
        Sexp::Atom(I(n)) => Expr::Num(i32::try_from(*n).unwrap()),
        Sexp::List(vec) => {
            match &vec[..] {
                [Sexp::Atom(S(op)), e] if op == "add1" => Expr::Add1(Box::new(parse_expr(e))),
                [Sexp::Atom(S(op)), e] if op == "sub1" => Expr::Sub1(Box::new(parse_expr(e))),
                [Sexp::Atom(S(op)), e] if op == "negate" => Expr::Negate(Box::new(parse_expr(e))),
                _ => panic!("parse error"),
            }
        },
        _ => panic!("parse error"),
    }
}

fn compile_expr(e: &Expr) -> String {
    match e {
        Expr::Num(n) => format!("mov rax, {}", *n),
        Expr::Add1(subexpr) => compile_expr(subexpr) + "\nadd rax, 1",
        Expr::Sub1(subexpr) => compile_expr(subexpr) + "\nsub rax, 1",
        &Expr::Negate(ref e) => {
            let sub_expr = compile_expr(e);
            format!("
                {sub_expr}
                neg rax
            ")
        },
    }
    
}

fn main() -> std::io::Result<()> {
    let args: Vec<String> = env::args().collect();

    let in_name = &args[1];
    let out_name = &args[2];

    let mut in_file = File::open(in_name)?;
    let mut in_contents = String::new();
    in_file.read_to_string(&mut in_contents)?;

    let expr = parse_expr(&parse(&in_contents).unwrap());
    let result = compile_expr(&expr);
    let asm_program = format!("
section .text
global our_code_starts_here
our_code_starts_here:
  {}
  ret
", result);

    let mut out_file = File::create(out_name)?;
    out_file.write_all(asm_program.as_bytes())?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn eval(expr: &Expr) -> i32 {
        match expr {
            Expr::Num(n) => *n,
            Expr::Add1(subexpr) => eval(subexpr) + 1,
            Expr::Sub1(subexpr) => eval(subexpr) - 1,
            Expr::Negate(subexpr) => -eval(subexpr),
        }
    }

    #[test]
    fn test1() {
        let input = "(add1 5)";
        let parsed = parse_expr(&parse(input).unwrap());
        assert_eq!(eval(&parsed), 6);
    }

    #[test]
    fn test2() {
        let input = "(sub1 10)";
        let parsed = parse_expr(&parse(input).unwrap());
        assert_eq!(eval(&parsed), 9);
    }

    #[test]
    fn test3() {
        let input = "(negate 42)";
        let parsed = parse_expr(&parse(input).unwrap());
        assert_eq!(eval(&parsed), -42);
    }

    #[test]
    fn test4() {
        let input = "(add1 (sub1 5))";
        let parsed = parse_expr(&parse(input).unwrap());
        assert_eq!(eval(&parsed), 5);
    }

    #[test]
    fn test5() {
        let input = "(negate (add1 (sub1 (negate 3))))";
        let parsed = parse_expr(&parse(input).unwrap());
        assert_eq!(eval(&parsed), 3);
    }
}

