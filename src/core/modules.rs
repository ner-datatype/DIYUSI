use super::ConstId;

use std::collections::HashMap;
use std::rc::{Rc, Weak};
use std::cell::RefCell;

#[derive(Clone, Debug)]
pub struct Scope {
    module: Rc<RefCell<Module>>,
    names: Vec<Name>,
    parent: Option< Rc<Scope> >,
}
impl Scope {
    pub fn top() -> Self {
        Scope {
            module: Rc::new(RefCell::new(Module::anonymous_top())),
            names: Vec::new(),
            parent: None,
        }
    }

    pub fn new(module: Rc<RefCell<Module>>, parent: Rc<Scope>) -> Self {
        Scope {
            module,
            names: Vec::new(),
            parent: Some(parent),
        }
    }

    pub fn resolve<'b, Q: IntoIterator<Item=&'b str> + Clone>(&self, qualifier: Q, identifier: &str) -> Option<ConstId>
    {
        search_const(&self.module, qualifier.clone(), identifier)
            .or_else(|| self.parent.clone().and_then(|parent| parent.resolve(qualifier, identifier)))
    }

    pub fn resolve_from_ident(&self, ident: &::syntax::ast::Ident) -> Option<ConstId> {
        self.resolve(ident.domain.iter().map(|s| s.as_str()), &ident.name)
    }

    pub fn module(&self) -> Rc<RefCell<Module>> { self.module.clone() }

    pub fn register_const<Q>(&mut self, qualifier: Q, identifier: String, cid: ConstId) -> Result<ConstId, String>
        where Q: IntoIterator<Item = String> + Clone
    {
        let qualifier: Vec<String> = qualifier.into_iter().collect();
        let module = search_module(&self.module, qualifier.iter().map(|s| s.as_str()))?;

        module.borrow_mut().register(cid, identifier.clone());
        self.names.push(Name{qualifier, identifier});

        Ok(cid)
    }

    pub fn resolve_namedhole(&self, name: &::syntax::ast::Ident) -> Option<usize>
    {
        let qualifier = name.domain.iter().map(|s| s.as_str());
        let identifier = &name.name;

        search_namedhole(&self.module, qualifier.clone(), identifier)
            .or_else(|| self.parent.clone().and_then(|parent| parent.resolve(qualifier, identifier)))
    }

    pub fn register_namedhole(&self, name: ::syntax::ast::Ident, id: usize)
        -> Result<(), (::core::errors::TranslateErr, ::syntax::Loc)>
    {
        let domain = name.domain;
        let loc = name.loc;

        let qualifier = domain.iter().map(|s| s.as_str());
        let module = search_module(&self.module, qualifier).map_err(
            |module_name| (::core::errors::TranslateErr::UndefinedModule{name: module_name.into()}, loc)
        )?;
        module.borrow_mut().register_namedhole(id, name.name);
        Ok(())
    }

    pub fn names(&self) -> &[Name] { &self.names }
}

#[derive(Clone, Debug)]
pub struct Name {
    pub qualifier: Vec<String>,
    pub identifier: String,
}
impl Name {
    pub fn simple(identifier: String) -> Self {
        Name{qualifier: Vec::new(), identifier}
    }
}
impl ::std::fmt::Display for Name {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        for qualifier in &self.qualifier {
            write!(f, "{}.", qualifier)?;
        }
        write!(f, "{}", self.identifier)
    }
}

#[derive(Clone, Debug)]
pub struct Module {
    name: String,
    ids: HashMap<String, ConstId>,
    namedhole_ids: HashMap<String, usize>,
    parent: Weak<RefCell<Module>>,
    children: HashMap< String, Rc<RefCell<Module>> >,
}
impl<'s> Module {
    pub fn anonymous_top() -> Self {
        Module {
            name: String::new(),
            ids: HashMap::new(),
            namedhole_ids: HashMap::new(),
            parent: Weak::new(),
            children: HashMap::new(),
        }
    }

    pub fn register(&mut self, id: ConstId, name: String) {
        self.ids.insert(name, id);
    }

    pub fn register_namedhole(&mut self, id: usize, name: String) {
        self.namedhole_ids.insert(name, id);
    }

    pub fn name(&self) -> &str { &self.name }
}

pub fn add_child<'s>(parent: &Rc<RefCell<Module>>, name: String) -> Result<Rc<RefCell<Module>>, ()> {
    let m = Rc::new( RefCell::new(
        Module {
            name: name.clone(),
            ids: HashMap::new(),
            namedhole_ids: HashMap::new(),
            parent: Rc::downgrade(parent),
            children: HashMap::new(),
        }
    ) );

    parent.borrow_mut().children.insert(name, m.clone())
        .map( |_| Err(()) ).unwrap_or( Ok(m) )
}

const PARENT_MODULE : &str = "super";
const SELF_MODULE : &str = "self";

pub fn search_module<'a, I>(this: &Rc<RefCell<Module>>, qualifier: I) -> Result<Rc<RefCell<Module>>, &'a str>
    where I: IntoIterator<Item=&'a str>
{
    let mut path = qualifier.into_iter();
    let next = path.next();
    match next {
        Some(x) if x == PARENT_MODULE => this.borrow().parent.upgrade().map(|x| search_module(&x, path))
            .unwrap_or(Err(PARENT_MODULE)),
        Some(x) if x == SELF_MODULE => search_module(this, path),
        Some(x) => this.borrow().children.get(x).map(|x| search_module(&x, path)).unwrap_or(Err(x)),
        None => Ok( this.clone() ),
    }
}

pub fn search_const<'a, Q: IntoIterator<Item=&'a str>>(this: &Rc<RefCell<Module>>, qualifier: Q, identifier: &str)
    -> Option<ConstId>
{
    search_module(this, qualifier).ok().and_then(|x| x.borrow().ids.get(identifier).cloned())
}

pub fn search_namedhole<'a, Q: IntoIterator<Item=&'a str>>(this: &Rc<RefCell<Module>>, qualifier: Q, identifier: &str)
    -> Option<ConstId>
{
    search_module(this, qualifier).ok().and_then(|x| x.borrow().namedhole_ids.get(identifier).cloned())
}