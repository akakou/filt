extern crate base64;
extern crate config;
extern crate serde_json;


/// State for scanner hit
pub enum IsHit {
    Err,
    Hit,
    Unhit
}

/// Result for Scannig
pub struct ScanResult {
    pub result: IsHit,
    pub messages: Vec<String>
}

impl ScanResult {
    /// Build new scan-result
    pub fn new() -> ScanResult {
        ScanResult {
            result: IsHit::Unhit,
            messages: Vec::new()
        }
    }

    /// Init new scanner by arguments
    pub fn init(is_hit: IsHit, messages: Vec<String>) -> ScanResult {
        ScanResult {
            result: is_hit,
            messages: messages
        }
    }

    /// Return json formated result
    pub fn to_string(&self) -> String {
        let is_hit = match self.result{
            IsHit::Err => "err",
            IsHit::Hit => "hit",
            IsHit::Unhit => "unhit"
        };

        let mut encoded_messages: Vec<String> = Vec::new();

        for message in &self.messages {
            let base64_message = base64::encode(message);
            encoded_messages.push(base64_message);        
        }
        
        let result = format!("{{\
                \"result\":\"{}\",\
                \"message\":{:?}\
            }}", is_hit, encoded_messages);
        
        return result;
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