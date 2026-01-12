use rand::Rng;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrowserFingerprint {
    pub user_agent: String,
    pub accept_language: String,
    pub accept_encoding: String,
    pub screen_resolution: String,
    pub timezone: String,
    pub webgl_vendor: String,
    pub webgl_renderer: String,
}

impl BrowserFingerprint {
    /// Generate a randomized but realistic browser fingerprint
    pub fn random() -> Self {
        let mut rng = rand::thread_rng();
        
        let user_agents = vec![
            "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36",
            "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36",
            "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36",
            "Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:121.0) Gecko/20100101 Firefox/121.0",
            "Mozilla/5.0 (Macintosh; Intel Mac OS X 10.15; rv:121.0) Gecko/20100101 Firefox/121.0",
        ];
        
        let languages = vec!["en-US,en;q=0.9", "en-GB,en;q=0.9", "en-US,en;q=0.5"];
        
        let resolutions = vec!["1920x1080", "2560x1440", "1366x768", "1536x864", "3840x2160"];
        
        let timezones = vec!["America/New_York", "America/Los_Angeles", "Europe/London", "Europe/Paris"];
        
        Self {
            user_agent: user_agents[rng.gen_range(0..user_agents.len())].to_string(),
            accept_language: languages[rng.gen_range(0..languages.len())].to_string(),
            accept_encoding: "gzip, deflate, br".to_string(),
            screen_resolution: resolutions[rng.gen_range(0..resolutions.len())].to_string(),
            timezone: timezones[rng.gen_range(0..timezones.len())].to_string(),
            webgl_vendor: "Google Inc. (NVIDIA)".to_string(),
            webgl_renderer: "ANGLE (NVIDIA, NVIDIA GeForce RTX 3070)".to_string(),
        }
    }
    
    /// Apply this fingerprint to HTTP request headers
    pub fn apply_to_headers(&self, headers: &mut hyper::HeaderMap) {
        headers.insert(
            hyper::header::USER_AGENT,
            self.user_agent.parse().unwrap(),
        );
        headers.insert(
            hyper::header::ACCEPT_LANGUAGE,
            self.accept_language.parse().unwrap(),
        );
        headers.insert(
            hyper::header::ACCEPT_ENCODING,
            self.accept_encoding.parse().unwrap(),
        );
    }
}

/// Canvas fingerprinting protection
#[derive(Clone)]
pub struct CanvasProtection {
    enabled: bool,
}

impl CanvasProtection {
    pub fn new(enabled: bool) -> Self {
        Self { enabled }
    }

    /// Add random noise to canvas data to prevent fingerprinting
    pub fn add_noise(data: &mut [u8]) {
        let mut rng = rand::thread_rng();
        
        // Add minimal noise that doesn't affect visual appearance
        for pixel in data.iter_mut().step_by(4) {
            if rng.gen_bool(0.01) {
                // Modify ~1% of pixels by ±1
                *pixel = pixel.saturating_add(if rng.gen_bool(0.5) { 1 } else { 255 });
            }
        }
    }

    /// Get JavaScript injection code to block canvas fingerprinting
    pub fn get_injection_script(&self) -> Option<String> {
        if !self.enabled {
            return None;
        }

        Some(r#"
<script>
(function() {
    'use strict';
    
    // Poison canvas fingerprinting
    const originalToDataURL = HTMLCanvasElement.prototype.toDataURL;
    const originalGetImageData = CanvasRenderingContext2D.prototype.getImageData;
    
    // Add noise to canvas data
    function addNoise(imageData) {
        const data = imageData.data;
        for (let i = 0; i < data.length; i += 4) {
            if (Math.random() < 0.01) {
                data[i] = (data[i] + (Math.random() > 0.5 ? 1 : -1)) & 0xff;
            }
        }
        return imageData;
    }
    
    // Override toDataURL
    HTMLCanvasElement.prototype.toDataURL = function() {
        const context = this.getContext('2d');
        if (context) {
            const imageData = context.getImageData(0, 0, this.width, this.height);
            addNoise(imageData);
            context.putImageData(imageData, 0, 0);
        }
        return originalToDataURL.apply(this, arguments);
    };
    
    // Override getImageData
    CanvasRenderingContext2D.prototype.getImageData = function() {
        const imageData = originalGetImageData.apply(this, arguments);
        return addNoise(imageData);
    };
    
    console.log('🛡️ Canvas fingerprinting protection active');
})();
</script>
"#.to_string())
    }

    pub fn is_enabled(&self) -> bool {
        self.enabled
    }
}
