
use std::collections::hash_map::HashMap;
use std::any::Any;
use std::sync::OnceLock;
use std::sync::Arc;
use std::sync::Mutex;

pub trait Lifecycle {
    fn on_create();
}

pub trait Init<C=Container> {

    fn new(container: &mut C) -> Self;
}

pub trait Singleton {
    fn id () -> &'static str;
    
    fn new(container: &mut Container) -> dyn Any + Send + Sync + Sized;
}

static singletons: OnceLock<HashMap<&'static str, Box<dyn Any + Sync + Send>>> = OnceLock::new();


pub struct System<T> where T: Init {
    value: T
}

impl<T> System<T> where T: Init {
    fn get_or_create(container: &mut Container) -> T {
        <T as Init>::new(container)
    }
}

pub struct SingletonSystem<T> where T: Singleton {
    value: OnceLock<T>
}

pub struct Container {
    singletons: HashMap<&'static str, Arc<Mutex<Box<dyn Any + Send + Sync>>>>
}

impl Container {

    fn new() -> Self {
        Self {
            singletons: HashMap::new()
        }
    }
    
    fn get_or_create<T: Init>(&mut self) -> T {
        <T as Init>::new(self)
    }
    
    fn get_or_create_singleton<S: Singleton + Send + Sync + 'static>(&mut self) -> Arc<Mutex<dyn Any + Send + Sync>> {
        if self.singletons.contains_key(<S as Singleton>::id()) {
            let any = self.singletons.get(<S as Singleton>::id()).unwrap();
            return any.clone();
        }
        let new = Arc::new(Mutex::new(Box::new(<S as Singleton>::new(self))));
        self.singletons.insert(<S as Singleton>::id(), new);
        let any = self.singletons.get(<S as Singleton>::id()).unwrap();
        any.clone()
    }
}
 

pub struct A(u32);

pub struct B(A, Arc<Mutex<C>>);

pub struct C(u32);

impl Init for C {
    fn new(c: &mut Container) -> Self {
        println!("This should happen only once.");
        C(2)
    }
}

impl Singleton for C {
    fn id() -> &'static str {
        "C"
    }
    
    fn new(c: &mut Container) -> dyn Any + Send + Sync + Sized {
        C(1)
    }
}

impl B {
    fn print(&self) {
        println!("output: {}, {}", self.0.0, self.1.0)
    }
    
    fn change_value(&mut self, value: u32) {
        self.0.0 = value;
    }
}

impl Init for A {
    fn new(container: &mut Container) -> Self {
        A(0)
    }
}

impl Init for B {
    fn new(container: &mut Container) -> Self {
        let a_instance: A = System::<A>::get_or_create(container);
        let c_instance = container.get_or_create_singleton();
        B(a_instance, c_instance)
    }
}

pub fn main() {
    let mut container = Container::new();
    let mut b_value: B = container.get_or_create();
    let b_value_2: B = container.get_or_create();
    b_value.print();
    b_value_2.print();
    
    b_value.change_value(32);
    b_value.print();
}
