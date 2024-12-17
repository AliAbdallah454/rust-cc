use std::fmt;
use serde::{Serialize,Deserialize};


#[derive(Serialize, Deserialize, Debug)]

pub struct Transaction<T: fmt::Display + PartialEq>
{
    pub source:T,
    pub destination:T,
    pub min:T,
    pub max:T,
    exception: bool
}

impl<T:PartialOrd +  fmt::Display > Transaction<T>
{
    pub fn new(source: T , destination: T, min: T , max: T)-> Self
    {
        Self {exception:(min > max ), source, destination,min,max }
    }
    pub fn in_range(&self,num: T) -> bool
    {
        if !self.exception
        {
            return num>self.min && num<=self.max;
        }
        return num<= self.max || num > self.min;
    }
    
}
impl<T:PartialOrd +  fmt::Display > fmt::Display for Transaction<T>
{
    fn fmt(&self,f: &mut fmt::Formatter)->fmt::Result
    {
        let stringy = match self.exception
        {
            true =>
            {
                format!("[0,{}] u ]{},..[",self.max,self.min)
            }
            ,
            _=>
            {
                format!("]{},{}]",self.min,self.max)
            }

        };
        write!(f,"source: {}  -  destination: {}  -  range: {}",self.source,self.destination,stringy)

    }
}
impl<T: fmt::Display + PartialEq> PartialEq for Transaction<T> {
    fn eq(&self, other: &Self) -> bool {
        self.source == other.source
            && self.destination == other.destination
            && self.exception == other.exception
    }
}