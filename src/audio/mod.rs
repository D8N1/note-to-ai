use crate::Result;
use std::path::PathBuf;
use std::time::Duration;
use anyhow::anyhow;

pub mod whisper;

/// Audio processing capabilities
pub struct AudioProcessor {
    whisper: Option<whisper::WhisperProcessor>,
}

impl AudioProcessor {
    pub async fn new() -> Result<Self> {
        let whisper = whisper::WhisperProcessor::new().await.ok();
        
        Ok(Self {
            whisper,
        })
    }
    
    /// Transcribe audio file to text
    pub async fn transcribe_audio(&self, audio_path: &PathBuf) -> Result<String> {
        if let Some(whisper) = &self.whisper {
            whisper.transcribe_file(audio_path).await
        } else {
            // Fallback for when whisper is not available
            Ok(format!("Audio transcription not available for: {}", audio_path.display()))
        }
    }
    
    /// Estimate audio duration from file
    pub async fn estimate_duration(&self, audio_path: &PathBuf) -> Result<Duration> {
        // Basic file size estimation (very rough)
        let metadata = std::fs::metadata(audio_path)
            .map_err(|e| anyhow!("Failed to read audio file metadata: {}", e))?;
        
        let file_size = metadata.len();
        
        // Rough estimation: 1MB ≈ 1 minute for typical voice recording
        let estimated_seconds = file_size / (1024 * 1024).max(1);
        
        Ok(Duration::from_secs(estimated_seconds.min(3600))) // Cap at 1 hour
    }
    
    /// Check if file is a supported audio format
    pub fn is_supported_format(&self, file_path: &PathBuf) -> bool {
        if let Some(extension) = file_path.extension() {
            if let Some(ext_str) = extension.to_str() {
                matches!(ext_str.to_lowercase().as_str(), 
                    "mp3" | "wav" | "m4a" | "flac" | "ogg" | "aac")
            } else {
                false
            }
        } else {
            false
        }
    }
}

/// Audio format detection
pub enum AudioFormat {
    Mp3,
    Wav,
    M4a,
    Flac,
    Ogg,
    Aac,
    Unknown,
}

impl AudioFormat {
    pub fn from_path(path: &PathBuf) -> Self {
        if let Some(extension) = path.extension() {
            if let Some(ext_str) = extension.to_str() {
                match ext_str.to_lowercase().as_str() {
                    "mp3" => AudioFormat::Mp3,
                    "wav" => AudioFormat::Wav,
                    "m4a" => AudioFormat::M4a,
                    "flac" => AudioFormat::Flac,
                    "ogg" => AudioFormat::Ogg,
                    "aac" => AudioFormat::Aac,
                    _ => AudioFormat::Unknown,
                }
            } else {
                AudioFormat::Unknown
            }
        } else {
            AudioFormat::Unknown
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[tokio::test]
    async fn test_audio_processor_creation() {
        let processor = AudioProcessor::new().await;
        assert!(processor.is_ok());
    }

    #[test]
    fn test_supported_format_detection() {
        let processor = AudioProcessor { whisper: None };
        
        assert!(processor.is_supported_format(&PathBuf::from("test.mp3")));
        assert!(processor.is_supported_format(&PathBuf::from("test.wav")));
        assert!(!processor.is_supported_format(&PathBuf::from("test.txt")));
    }

    #[test]
    fn test_audio_format_detection() {
        assert!(matches!(AudioFormat::from_path(&PathBuf::from("test.mp3")), AudioFormat::Mp3));
        assert!(matches!(AudioFormat::from_path(&PathBuf::from("test.wav")), AudioFormat::Wav));
        assert!(matches!(AudioFormat::from_path(&PathBuf::from("test.txt")), AudioFormat::Unknown));
    }
}
