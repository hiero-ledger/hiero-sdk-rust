use sha3::{
    Digest,
    Keccak256,
};

#[derive(Debug, Clone)]
pub struct ContractFunctionSelector {
    digest: Option<Keccak256>,
    needs_comma: bool,
    finished_bytes: Option<[u8; 4]>,
}

impl From<[u8; 4]> for ContractFunctionSelector {
    fn from(value: [u8; 4]) -> Self {
        Self { digest: None, needs_comma: false, finished_bytes: Some(value) }
    }
}

impl ContractFunctionSelector {
    #[allow(dead_code)]
    pub fn new(func_name: &str) -> Self {
        let mut digest = <Keccak256 as Digest>::new_with_prefix(func_name.as_bytes());
        digest.update("(".as_bytes());
        ContractFunctionSelector { digest: Some(digest), needs_comma: false, finished_bytes: None }
    }

    #[allow(dead_code)]
    pub(crate) fn add_param_type(&mut self, param_type_name: &str) -> &mut Self {
        if let Some(digest) = &mut self.digest {
            if self.needs_comma {
                digest.update(",".as_bytes())
            }
            digest.update(param_type_name.as_bytes());
            self.needs_comma = true;
        }
        self
    }

    #[allow(dead_code)]
    pub fn finish(&mut self) -> [u8; 4] {
        if let Some(mut digest) = self.digest.take() {
            digest.update(")".as_bytes());
            self.finished_bytes = Some(digest.finalize()[0..4].try_into().unwrap());
        }
        self.finished_bytes.clone().unwrap()
    }

    #[allow(dead_code)]
    pub fn add_string(&mut self) -> &mut Self {
        self.add_param_type("string")
    }

    #[allow(dead_code)]
    pub fn add_string_array(&mut self) -> &mut Self {
        self.add_param_type("string[]")
    }

    #[allow(dead_code)]
    pub fn add_bytes(&mut self) -> &mut Self {
        self.add_param_type("bytes")
    }

    #[allow(dead_code)]
    pub fn add_bytes_array(&mut self) -> &mut Self {
        self.add_param_type("bytes[]")
    }

    #[allow(dead_code)]
    pub fn add_bytes32(&mut self) -> &mut Self {
        self.add_param_type("bytes32")
    }

    #[allow(dead_code)]
    pub fn add_bytes32_array(&mut self) -> &mut Self {
        self.add_param_type("bytes32[]")
    }

    #[allow(dead_code)]
    pub fn add_bool(&mut self) -> &mut Self {
        self.add_param_type("bool")
    }

    #[allow(dead_code)]
    pub fn add_int8(&mut self) -> &mut Self {
        self.add_param_type("int8")
    }

    #[allow(dead_code)]
    pub fn add_int16(&mut self) -> &mut Self {
        self.add_param_type("int16")
    }

    #[allow(dead_code)]
    pub fn add_int32(&mut self) -> &mut Self {
        self.add_param_type("int32")
    }

    #[allow(dead_code)]
    pub fn add_int64(&mut self) -> &mut Self {
        self.add_param_type("int64")
    }

    #[allow(dead_code)]
    pub fn add_int256(&mut self) -> &mut Self {
        self.add_param_type("int256")
    }

    #[allow(dead_code)]
    pub fn add_uint8(&mut self) -> &mut Self {
        self.add_param_type("uint8")
    }

    #[allow(dead_code)]
    pub fn add_uint16(&mut self) -> &mut Self {
        self.add_param_type("uint16")
    }

    #[allow(dead_code)]
    pub fn add_uint32(&mut self) -> &mut Self {
        self.add_param_type("uint32")
    }

    #[allow(dead_code)]
    pub fn add_uint64(&mut self) -> &mut Self {
        self.add_param_type("uint64")
    }

    #[allow(dead_code)]
    pub fn add_uint256(&mut self) -> &mut Self {
        self.add_param_type("uint256")
    }

    #[allow(dead_code)]
    pub fn add_int8_array(&mut self) -> &mut Self {
        self.add_param_type("int8[]")
    }

    #[allow(dead_code)]
    pub fn add_int16_array(&mut self) -> &mut Self {
        self.add_param_type("int16[]")
    }

    #[allow(dead_code)]
    pub fn add_int32_array(&mut self) -> &mut Self {
        self.add_param_type("int32[]")
    }

    #[allow(dead_code)]
    pub fn add_int64_array(&mut self) -> &mut Self {
        self.add_param_type("int64[]")
    }

    #[allow(dead_code)]
    pub fn add_int256_array(&mut self) -> &mut Self {
        self.add_param_type("int256[]")
    }

    #[allow(dead_code)]
    pub fn add_uint8_array(&mut self) -> &mut Self {
        self.add_param_type("uint8[]")
    }

    #[allow(dead_code)]
    pub fn add_uint16_array(&mut self) -> &mut Self {
        self.add_param_type("uint16[]")
    }

    #[allow(dead_code)]
    pub fn add_uint32_array(&mut self) -> &mut Self {
        self.add_param_type("uint32[]")
    }

    #[allow(dead_code)]
    pub fn add_uint64_array(&mut self) -> &mut Self {
        self.add_param_type("uint64[]")
    }

    #[allow(dead_code)]
    pub fn add_uint256_array(&mut self) -> &mut Self {
        self.add_param_type("uint256[]")
    }

    #[allow(dead_code)]
    pub fn add_address(&mut self) -> &mut Self {
        self.add_param_type("address")
    }

    #[allow(dead_code)]
    pub fn add_address_array(&mut self) -> &mut Self {
        self.add_param_type("address[]")
    }

    #[allow(dead_code)]
    pub fn add_function(&mut self) -> &mut Self {
        self.add_param_type("function")
    }
}
