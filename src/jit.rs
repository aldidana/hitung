use std::collections::HashMap;
use std::path::Path;

use inkwell;
use inkwell::builder::Builder;
use inkwell::context::Context;
use inkwell::module::Module;
use inkwell::values::PointerValue;
use inkwell::FloatPredicate;
use inkwell::OptimizationLevel;

use crate::expression::Expression;
use crate::lexer::Lexer;
use crate::parser::Parser;
use crate::token::Token;

pub type FuncSign = unsafe extern "C" fn() -> f64;

pub struct Compiler<'ctx> {
    context: &'ctx Context,
    module: Module<'ctx>,
    builder: Builder<'ctx>,

    variables: HashMap<String, PointerValue<'ctx>>,
    debug: bool,
}

impl<'ctx> Compiler<'ctx> {
    pub fn new(context: &'ctx Context, debug: bool) -> Self {
        let module = context.create_module("hitung");
        let builder = context.create_builder();

        Compiler {
            context,
            module,
            builder,
            variables: HashMap::new(),
            debug,
        }
    }
    pub fn compile_source(&mut self, source: &str) -> Result<f64, String> {
        let lexer = Lexer::new(source);
        let tokens = lexer.lex();
        let mut parser = Parser::new(tokens);
        match parser.expr(0) {
            Ok(expression) => self.jit_compile(expression),
            Err(e) => Err(e),
        }
    }

    pub fn jit_compile(&mut self, expr: Expression) -> Result<f64, String> {
        let float = self.context.f64_type();
        let fn_type = float.fn_type(&[], false);
        let function = self.module.add_function("berhitung", fn_type, None);
        let basic_block = self.context.append_basic_block(function, "entry");
        self.builder.position_at_end(basic_block);

        let return_val = self.eval(expr)?;
        self.builder.build_return(Some(&return_val));

        let execution_engine = self
            .module
            .create_jit_execution_engine(OptimizationLevel::None)
            .map_err(|e| e.to_string())?;

        let last_func = self.module.get_last_function().expect("Error when get last function");
        let last_func_name = last_func.get_name().to_str().expect("Error when get last function name");

        let function_calc = unsafe { execution_engine.get_function::<FuncSign>(last_func_name) };

        if self.debug {
            println!("LLVM IR:");
            function.print_to_stderr();
            self.module
                .print_to_file(Path::new("hitung.ll"))
                .expect("Error print to file");
        }

        match execution_engine.remove_module(&self.module) {
            Ok(_ok) => match function_calc {
                Ok(f) => Ok(unsafe { f.call() }),
                Err(err) => {
                    println!("!> Error during execution: {:?}", err);
                    Err(err.to_string())
                }
            },
            Err(err) => Err(err.to_string()),
        }
    }

    fn eval(
        &mut self,
        expression: Expression,
    ) -> Result<inkwell::values::FloatValue<'ctx>, String> {
        match expression {
            Expression::Variable(name) => match self.variables.get(&name) {
                Some(value) => {
                    let val = self.builder.build_load(*value, name.as_str());
                    Ok(val.into_float_value())
                }
                None => Err("Variable not declared".to_string()),
            },
            Expression::Num(n) => {
                let float = self.context.f64_type();
                Ok(float.const_float(n as f64))
            }
            Expression::Unary(operator, expr) => match operator {
                Token::Add => {
                    let num = self.eval(*expr)?;

                    Ok(num)
                },
                Token::Sub => {
                    let float = self.context.f64_type();
                    let num = self.eval(*expr)?;
                    let rhs = float.const_float_from_string("-1");
                    let sum = self.builder.build_float_mul(num, rhs, "mul");
                    Ok(sum)
                }
                _ => Err("Expression for Unary must be + or -".to_string()),
            },
            Expression::Binary(left, operator, right) => match operator {
                Token::LT | Token::GT => {
                    let lhs = self.eval(*left)?;
                    let rhs = self.eval(*right)?;

                    let predicate = match operator {
                        Token::LT => FloatPredicate::OLT,
                        Token::GT => FloatPredicate::OGT,
                        _ => return Err("Operator not supported".to_string()),
                    };

                    let conditional = self.builder.build_float_compare(predicate, lhs, rhs, "if");

                    Ok(self.builder.build_unsigned_int_to_float(
                        conditional,
                        self.context.f64_type(),
                        "bool",
                    ))
                },
                Token::ASSIGN => match *left {
                    Expression::Variable(var) => {
                        let f64_type = self.context.f64_type();
                        let global = self.module.add_global(f64_type, None, var.as_str());
                        global.set_initializer(&f64_type.const_float(0 as f64));
                        let rhs = self.eval(*right)?;
                        global.set_initializer(&rhs);
                        self.builder.build_store(global.as_pointer_value(), rhs);

                        let value = self
                            .builder
                            .build_load(global.as_pointer_value(), var.as_str());
                        self.variables.insert(var, global.as_pointer_value());

                        Ok(value.into_float_value())
                    }
                    _ => Err("Assignment must be a variable".to_string()),
                },
                _ => {
                    let lhs = self.eval(*left)?;
                    let rhs = self.eval(*right)?;

                    match operator {
                        Token::Add => Ok(self.builder.build_float_add(lhs, rhs, "add")),
                        Token::Sub => Ok(self.builder.build_float_sub(lhs, rhs, "sub")),
                        Token::Mul => Ok(self.builder.build_float_mul(lhs, rhs, "mul")),
                        Token::Div => Ok(self.builder.build_float_div(lhs, rhs, "div")),
                        _ => Err("Operator not supported".to_string()),
                    }
                }
            },
            Expression::Paren(expr) => {
                let result = &self.eval(*expr)?;
                Ok(*result)
            }
            Expression::Conditional(cond, then, els) => {
                let if_cond = self.eval(*cond)?;

                let function = self.module.get_last_function().expect("Error when get last function");

                let then_block = self.context.append_basic_block(function, "entry");
                let else_block = self.context.append_basic_block(function, "entry");
                let cont_block = self.context.append_basic_block(function, "entry");

                let i64_type = self.context.i64_type();

                self.builder.build_conditional_branch(
                    if_cond.const_to_unsigned_int(i64_type),
                    then_block,
                    else_block,
                );
                self.builder.position_at_end(then_block);

                let then_value = self.eval(*then)?;
                self.builder.build_unconditional_branch(cont_block);
                let then_block = self.builder.get_insert_block().expect("Error when get insert block");

                // build else block
                self.builder.position_at_end(else_block);
                let else_value = self.eval(*els)?;
                self.builder.build_unconditional_branch(cont_block);

                let else_block = self.builder.get_insert_block().expect("Error when get insert block");
                self.builder.position_at_end(cont_block);

                let phi = self.builder.build_phi(self.context.f64_type(), "entry");
                phi.add_incoming(&[(&then_value, then_block), (&else_value, else_block)]);

                Ok(phi.as_basic_value().into_float_value())
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_eval_from_expression() {
        let expression = Expression::Binary(
            Box::new(Expression::Binary(
                Box::new(Expression::from(3)),
                Token::Add,
                Box::new(Expression::from(2)),
            )),
            Token::Sub,
            Box::new(Expression::from(2)),
        );

        let context = Context::create();
        let module = context.create_module("test_hitung");
        let builder = context.create_builder();

        let mut compiler = Compiler {
            context: &context,
            module,
            builder,
            variables: Default::default(),
            debug: false,
        };

        let actual = compiler.jit_compile(expression).unwrap();
        assert_eq!(3.0, actual);
    }

    #[test]
    fn test_eval_from_source() {
        let context = Context::create();
        let mut compiler = Compiler::new(&context, false);

        let actual = compiler.compile_source(r"2 + 2 * 3 / 2").unwrap();

        assert_eq!(5.0, actual);
    }

    #[test]
    fn test_eval_from_source_if_then_else() {
        let context = Context::create();
        let mut compiler = Compiler::new(&context, false);

        let actual = compiler.compile_source(r"if 1 < 2 then 123 else 456").unwrap();

        assert_eq!(123.0, actual);
    }
}
