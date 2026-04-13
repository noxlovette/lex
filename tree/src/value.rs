use crate::{
    Callable, Environment, Interpreter, Literal, RuntimeControl, RuntimeError, RuntimeResult, Stmt,
    Token,
};
use std::{
    cell::RefCell,
    cmp::Ordering,
    collections::HashMap,
    fmt::{Display, Formatter},
    ops::{Add, Div, Mul, Neg, Not, Sub},
    rc::Rc,
    time::UNIX_EPOCH,
};

#[derive(PartialEq, Debug, Clone, Default)]
pub enum Value {
    #[default]
    Nil,
    Bool(bool),
    Number(f64),
    String(String),
    Native(NativeFunction),
    Function(Function),
    Class(Class),
    Instance(Rc<RefCell<Instance>>),
}

#[derive(Clone, Debug, PartialEq)]
pub struct Class {
    name: String,
    superclass: Option<Rc<Class>>,
    methods: HashMap<String, Function>,
}

impl Class {
    pub fn new(
        name: impl Into<String>,
        superclass: Option<Rc<Class>>,
        methods: HashMap<String, Function>,
    ) -> Self {
        Self {
            name: name.into(),
            superclass,
            methods,
        }
    }

    pub fn find_method(&self, name: &str) -> Option<Function> {
        self.methods.get(name).cloned().or_else(|| {
            self.superclass
                .as_ref()
                .and_then(|class| class.find_method(name))
        })
    }
}

impl Callable for Class {
    fn call(self, interpreter: &mut Interpreter, args: Vec<Value>) -> RuntimeResult<Value> {
        let instance = Rc::new(RefCell::new(Instance::new(self.clone())));

        if let Some(initializer) = self.find_method("init") {
            initializer.bind(&instance).call(interpreter, args)?;
        }

        Ok(Value::Instance(instance))
    }

    fn arity(&self) -> usize {
        self.find_method("init").map_or(0, |init| init.arity())
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Instance {
    class: Class,
    fields: HashMap<String, Value>,
}

impl Instance {
    pub fn new(class: Class) -> Self {
        Self {
            class,
            fields: HashMap::new(),
        }
    }

    pub fn get(instance: &Rc<RefCell<Self>>, name: &Token) -> RuntimeResult<Value> {
        if let Some(value) = instance.borrow().fields.get(&name.lexeme).cloned() {
            return Ok(value);
        }

        let class = instance.borrow().class.clone();
        if let Some(method) = class.find_method(&name.lexeme) {
            return Ok(Value::Function(method.bind(instance)));
        }

        Err(RuntimeError::Undefined {
            lexeme: name.lexeme.clone(),
        })
    }

    pub fn set(&mut self, name: &Token, value: Value) {
        self.fields.insert(name.lexeme.clone(), value);
    }
}

#[derive(PartialEq, Debug, Clone, Default)]
pub struct NativeFunction {
    arity: usize,
}

impl NativeFunction {
    pub fn new(arity: usize) -> Self {
        Self { arity }
    }
}

#[derive(PartialEq, Debug, Clone)]
pub struct Function {
    pub declaration: Stmt,
    pub closure: Rc<RefCell<Environment>>,
    pub is_initializer: bool,
}

impl Function {
    pub fn bind(&self, instance: &Rc<RefCell<Instance>>) -> Function {
        let environment = Environment::new().with_enclosing(self.closure.clone()).rc();
        environment
            .borrow_mut()
            .define("this".to_string(), Some(Value::Instance(instance.clone())));

        Self {
            declaration: self.declaration.clone(),
            closure: environment,
            is_initializer: self.is_initializer,
        }
    }
}

impl Callable for Function {
    fn arity(&self) -> usize {
        match &self.declaration {
            Stmt::Function { params, .. } => params.len(),
            _ => unreachable!(),
        }
    }

    fn call(self, interpreter: &mut Interpreter, args: Vec<Value>) -> RuntimeResult<Value> {
        match &self.declaration {
            Stmt::Function { params, body, .. } => {
                let environment = Environment::new().with_enclosing(self.closure.clone()).rc();

                for (index, param) in params.iter().enumerate() {
                    environment
                        .borrow_mut()
                        .define(param.lexeme.clone(), args.get(index).cloned());
                }

                let result = interpreter.execute_block(body, environment);

                match result {
                    Ok(()) => {
                        if self.is_initializer {
                            self.closure.borrow().get_at(0, "this")
                        } else {
                            Ok(Value::Nil)
                        }
                    }
                    Err(RuntimeControl::Return(value)) => {
                        if self.is_initializer {
                            self.closure.borrow().get_at(0, "this")
                        } else {
                            Ok(*value)
                        }
                    }
                    Err(RuntimeControl::Error(error)) => Err(*error),
                }
            }
            _ => Err(RuntimeError::NotCallable(
                "Tried to call a non-function value".to_string(),
            )),
        }
    }
}

impl Display for NativeFunction {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "<native fn>")
    }
}

impl Display for Function {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match &self.declaration {
            Stmt::Function { name, .. } => write!(f, "<fn {}>", name.lexeme),
            _ => unreachable!(),
        }
    }
}

impl Display for Class {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}

impl Display for Instance {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} instance", self.class)
    }
}

impl Callable for NativeFunction {
    fn call(
        self,
        _interpreter: &mut crate::Interpreter,
        _args: Vec<Value>,
    ) -> RuntimeResult<Value> {
        Ok(Value::Number(
            std::time::SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("system time should be after the unix epoch")
                .as_secs_f64(),
        ))
    }

    fn arity(&self) -> usize {
        self.arity
    }
}

impl Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use Value::*;

        match self {
            Number(n) => {
                let text = n.to_string();
                write!(f, "{}", text.trim_end_matches(".0"))
            }
            String(s) => write!(f, "{s}"),
            Bool(b) => write!(f, "{b}"),
            Nil => write!(f, "nil"),
            Native(n) => write!(f, "{n}"),
            Function(function) => write!(f, "{function}"),
            Class(class) => write!(f, "{class}"),
            Instance(instance) => write!(f, "{}", instance.borrow()),
        }
    }
}

impl From<&Literal> for Value {
    fn from(value: &Literal) -> Self {
        use Literal::*;

        match value {
            Nil => Self::Nil,
            Bool(b) => Self::Bool(*b),
            Number(n) => Self::Number(*n),
            String(s) => Self::String(s.clone()),
        }
    }
}

impl Not for Value {
    type Output = Self;

    fn not(self) -> Self::Output {
        Self::Bool(!self.is_truthy())
    }
}

impl Neg for Value {
    type Output = RuntimeResult<Self>;

    fn neg(self) -> Self::Output {
        match self {
            Self::Number(value) => Ok(Value::Number(-value)),
            _ => Err(RuntimeError::TypeError {
                message: "Tried to apply '-' operator on a non-number".to_string(),
                value: Box::new(self),
            }),
        }
    }
}

impl From<&Literal> for RuntimeResult<Value> {
    fn from(value: &Literal) -> Self {
        Ok(value.into())
    }
}

pub trait IsTruthy {
    fn is_truthy(&self) -> bool;
}

impl IsTruthy for Value {
    fn is_truthy(&self) -> bool {
        match self {
            Value::Nil => false,
            Value::Bool(value) => *value,
            _ => true,
        }
    }
}

impl Add for Value {
    type Output = RuntimeResult<Self>;

    fn add(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Self::Number(left), Self::Number(right)) => Ok(Self::Number(left + right)),
            (Self::String(left), Self::String(right)) => Ok(Self::String(left + &right)),
            (left, _) => Err(RuntimeError::TypeError {
                message: "Operands must be either strings or numbers".to_string(),
                value: Box::new(left),
            }),
        }
    }
}

impl PartialOrd for Value {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        match (self, other) {
            (Value::Number(left), Value::Number(right)) => left.partial_cmp(right),
            _ => None,
        }
    }
}

impl From<bool> for Value {
    fn from(value: bool) -> Self {
        Self::Bool(value)
    }
}

impl From<String> for Value {
    fn from(value: String) -> Self {
        Self::String(value)
    }
}

impl From<&str> for Value {
    fn from(value: &str) -> Self {
        Self::String(value.to_string())
    }
}

impl From<f64> for Value {
    fn from(value: f64) -> Self {
        Self::Number(value)
    }
}

impl<T> From<Option<T>> for Value
where
    T: Into<Value>,
{
    fn from(value: Option<T>) -> Self {
        match value {
            Some(value) => value.into(),
            None => Value::Nil,
        }
    }
}

impl From<()> for Value {
    fn from(_: ()) -> Self {
        Value::Nil
    }
}

macro_rules! impl_numeric_binop {
    ($trait:ident, $method:ident, $op:tt) => {
        impl $trait for Value {
            type Output = RuntimeResult<Self>;

            fn $method(self, rhs: Self) -> Self::Output {
                match (self, rhs) {
                    (Self::Number(left), Self::Number(right)) => Ok(Self::Number(left $op right)),
                    (left, _) => Err(RuntimeError::TypeError {
                        message: "Operands must be numbers".to_string(),
                        value: Box::new(left),
                    }),
                }
            }
        }
    };
}

impl_numeric_binop!(Sub, sub, -);
impl_numeric_binop!(Div, div, /);
impl_numeric_binop!(Mul, mul, *);
