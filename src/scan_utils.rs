extern crate base64;
extern crate config;
extern crate serde_json;



/// Result for Scannig
pub struct ScanResult {
    pub hit: bool,
    pub success: bool,
    pub messages: Vec<String>
}

impl ScanResult {
    /// Build new scan-result
    pub fn new() -> ScanResult {
        ScanResult {
            hit: false,
            success: true,
            messages: Vec::new()
        }
    }

    /// Init new scanner by arguments
    pub fn init(_hit: bool, success: bool, messages: Vec<String>) -> ScanResult {
        ScanResult {
            hit: _hit,
            success: true,
            messages: messages
        }
    }

    /// Return json formated result
    pub fn to_string(&self) -> String {
        let mut encoded_messages: Vec<String> = Vec::new();

        for message in &self.messages {
            let base64_message = base64::encode(message);
            encoded_messages.push(base64_message);        
        }
        
        let result = format!("{{\
                \"hit\":{},\
                \"sucess\":{},\
                \"message\":{:?}\
            }}", self.hit, self.success, encoded_messages);
        
        return result;
    }

    /// Add results and 
    pub fn add(&mut self, result: &mut  ScanResult) {
        self.hit = self.hit || result.hit;
        self.messages.append(&mut result.messages);
    }
}

/// Scan target
pub struct ScanTarget {
    pub target: String,
    pub param: serde_json::value::Value
}

impl ScanTarget {
    /// Build new scan-target
    pub fn new(param: serde_json::value::Value) -> Result<ScanTarget, String> {
        // check has target
        let mut target = match param.get("target") {
            Some(_target) => _target.to_string(),
            None => {
                return Err("target not found".to_string());
            }
        };

        target.pop();
        target.remove(0);

        Ok(
            ScanTarget {
                target: target,
                param: param
            }
        )
    }
}