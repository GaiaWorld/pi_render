
pub type KeyShaderDefines = u128;

#[derive(Debug, Clone, Default)]
pub struct ShaderDefinesSet {
    pub list: Vec<pi_atom::Atom>,
}
impl ShaderDefinesSet {
    pub fn get(&self, key: KeyShaderDefines) -> naga::FastHashMap<String, String> {
        let mut result = naga::FastHashMap::<String, String>::default();

        let len = self.list.len();
        for i in 0..len {
            if (1 << i) & key > 0 {
                let val = self.list.get(i).unwrap();
                result.insert(String::from(val.as_str()), String::from(val.as_str()));
            } 
        }

        result
    }
}
impl From<(&Vec<pi_atom::Atom>, &Vec<pi_atom::Atom>)> for ShaderDefinesSet {
    fn from(value: (&Vec<pi_atom::Atom>, &Vec<pi_atom::Atom>)) -> Self {
        let mut result = Self::default();

        value.0.iter().for_each(|val| {
            match result.list.binary_search(val) {
                Ok(_) => {},
                Err(index) => { result.list.insert(index, val.clone()) },
            }
        });

        value.1.iter().for_each(|val| {
            match result.list.binary_search(val) {
                Ok(_) => {},
                Err(index) => { result.list.insert(index, val.clone()) },
            }
        });

        result
    }
}