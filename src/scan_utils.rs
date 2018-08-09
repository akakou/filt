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
    pub message: String
}

impl ScanResult {
    /// Build new scan-result
    pub fn new() -> ScanResult {
        ScanResult {
            result: IsHit::Unhit,
            message: "".to_string()
        }
    }

    /// Init new scanner by arguments
    pub fn init(is_hit: IsHit, message: String) -> ScanResult {
        ScanResult {
            result: is_hit,
            message: message
        }
    }

    /// Return json formated result
    pub fn to_string(&self) -> String {
        let is_hit = match self.result{
            IsHit::Err => "err",
            IsHit::Hit => "hit",
            IsHit::Unhit => "unhit"
        };
        
        let base64_message = base64::encode(&self.message);
        
        let result = format!("{{\
                'result':'{}',\
                'message':'{}'\
            }}", is_hit, base64_message);
        
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
        let target = match param.get("target") {
            Some(_target) => _target.to_string(),
            None => {
                return Err("target not found".to_string());
            }
        };

        Ok(
            ScanTarget {
                target: target,
                param: param
            }
        )
    }
}