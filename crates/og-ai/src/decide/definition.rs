//! Named behaviour input, shared by author declarations and runtime creation.
//! The author supplies canonical refs; runtime creation supplies cooked IDs.
#[cfg(feature = "source")]
use og_core::reflect::source::schemars::JsonSchema;
use serde::Deserialize;
use std::collections::BTreeMap;

macro_rules! shapes {
    ($($item:item)*) => {$(
        #[derive(Clone, Debug, Deserialize)]
        #[cfg_attr(feature = "source", derive(JsonSchema))]
        #[cfg_attr(feature = "source", schemars(crate = "og_core::reflect::source::schemars"))]
        $item
    )*};
}
shapes! {
    #[serde(untagged)]
    pub enum Value { Number(f64), Bool(bool), Vector([f32;3]) }
    #[serde(untagged)]
    pub enum Speed { Fixed(f32), Read(String) }
    #[serde(rename_all="camelCase",deny_unknown_fields)]
    pub struct Behaviour<R=String> {
        pub name:String,
        #[serde(default)] pub keys:BTreeMap<String,Value>,
        #[serde(default)] pub routes:BTreeMap<String,Vec<[f32;3]>>,
        pub root:Root<R>,
    }
    #[serde(tag="kind",rename_all="kebab-case",deny_unknown_fields)]
    pub enum Root<R=String> {
        Tree {node:Node<R>},
        Machine {initial:String,states:Vec<State<R>>},
    }
    #[serde(deny_unknown_fields)]
    pub struct State<R=String> {
        pub name:String,
        #[serde(default)] pub enter:Vec<Action>,
        #[serde(default)] pub exit:Vec<Action>,
        pub body:Option<Node<R>>,
        #[serde(default)] pub transitions:Vec<Transition>,
    }
    #[serde(deny_unknown_fields)]
    pub struct Transition {
        pub to:String,
        #[serde(default)] pub checks:Vec<Check>,
        #[cfg_attr(feature="source",schemars(range(min=0),extend("x-og-unit"="s")))]
        pub after:Option<f32>,
    }
    #[serde(deny_unknown_fields)]
    pub struct Check {
        pub operand:String,
        pub op:Op,
        pub value:Option<Value>,
        /// Dynamic comparison operand, exclusive with value.
        pub against:Option<String>,
    }
    #[serde(rename_all="camelCase")]
    pub enum Op { Is,Not,Below,Above,AtMost,AtLeast,Truthy }
    #[serde(tag="kind",rename_all="kebab-case",deny_unknown_fields)]
    pub enum Node<R=String> {
        Selector {name:Option<String>,children:Vec<Node<R>>},
        Sequence {name:Option<String>,children:Vec<Node<R>>},
        Steps {name:Option<String>,children:Vec<Node<R>>},
        Parallel {name:Option<String>,all:bool,children:Vec<Node<R>>},
        Invert {name:Option<String>,child:Box<Node<R>>},
        Succeed {name:Option<String>,child:Box<Node<R>>},
        Repeat {name:Option<String>,times:u32,child:Box<Node<R>>},
        Cooldown {
            name:Option<String>,
            #[cfg_attr(feature="source",schemars(range(min=0),extend("x-og-unit"="s")))]
            seconds:f32,child:Box<Node<R>>,
        },
        Timeout {
            name:Option<String>,
            #[cfg_attr(feature="source",schemars(range(min=0),extend("x-og-unit"="s")))]
            seconds:f32,child:Box<Node<R>>,
        },
        Check {name:Option<String>,check:Check},
        Action {name:Option<String>,action:Action},
        Subtree {
            name:Option<String>,
            #[cfg_attr(feature="source",schemars(extend("x-og-ref-kinds"=["behaviour"])))]
            source:R,
        },
        Utility {name:Option<String>,inertia:f32,options:Vec<UtilityOption<R>>},
    }
    #[serde(deny_unknown_fields)]
    pub struct UtilityOption<R=String> {
        pub name:String,
        #[cfg_attr(feature="source",schemars(length(min=1)))]
        pub scores:Vec<Consideration>,
        #[cfg_attr(feature="source",schemars(length(min=1)))]
        pub children:Vec<Node<R>>,
    }
    #[serde(tag="kind",rename_all="kebab-case",deny_unknown_fields)]
    pub enum Consideration {
        Constant {value:f32},
        Read {operand:String,curve:Curve,lo:f32,hi:f32},
    }
    #[serde(rename_all="lowercase")]
    pub enum Curve {Linear,Inverse,Quadratic,Sqrt,Smooth,Step}
    #[serde(tag="kind",rename_all="kebab-case",deny_unknown_fields)]
    pub enum Action {
        MoveTo {key:Option<String>,point:Option<[f32;3]>,speed:Option<Speed>},
        Chase {key:String,speed:Option<Speed>,reach:f32},
        Flee {key:String,speed:Option<Speed>,distance:f32},
        Patrol {route:String,wait:f32,speed:Option<Speed>,loops:bool},
        Wander {radius:f32,speed:Option<Speed>,wait:f32},
        Wait {
            #[cfg_attr(feature="source",schemars(range(min=0),extend("x-og-unit"="s")))]
            seconds:f32,
        },
        Stop,
        Set {key:String,value:Value},
        Command {name:String,args:Vec<String>,cooldown:f32},
        Game {name:String,args:Vec<String>},
    }
}
