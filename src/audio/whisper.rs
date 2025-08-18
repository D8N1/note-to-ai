use crate::Result;
use std::path::PathBuf;
use std::process::Command;
use anyhow::{anyhow, Context};
use tokio::fs;

/// Whisper audio transcription processor
pub struct WhisperProcessor {
    model_path: Option<PathBuf>,
    whisper_cpp_path: Option<PathBuf>,
}

impl WhisperProcessor {
    /// Create new Whisper processor
    pub async fn new() -> Result<Self> {
        let mut processor = Self {
            model_path: None,
            whisper_cpp_path: None,
        };
        
        // Try to locate whisper model
        processor.locate_whisper_resources().await?;
        
        Ok(processor)
    }
    
    /// Transcribe audio file to text
    pub async fn transcribe_file(&self, audio_path: &PathBuf) -> Result<String> {
        // Check if audio file exists
        if !audio_path.exists() {
            return Err(anyhow!("Audio file not found: {}", audio_path.display()).into());
        }
        
        // If we have whisper.cpp available, use it
        if let (Some(whisper_path), Some(model_path)) = (&self.whisper_cpp_path, &self.model_path) {
            self.transcribe_with_whisper_cpp(audio_path, whisper_path, model_path).await
        } else {
            // Fallback to mock transcription for development
            self.mock_transcription(audio_path).await
        }
    }
    
    /// Transcribe audio data from memory
    pub async fn transcribe_audio(&self, audio_path: &PathBuf) -> Result<String> {
        self.transcribe_file(audio_path).await
    }
    
    /// Locate Whisper resources on the system
    async fn locate_whisper_resources(&mut self) -> Result<()> {
        // Look for whisper.cpp in common locations
        let whisper_paths = [
            PathBuf::from("./models/whisper.cpp/main"),
            PathBuf::from("./whisper.cpp/main"),
            PathBuf::from("/usr/local/bin/whisper"),
            PathBuf::from("/opt/homebrew/bin/whisper"),
        ];
        
        for path in &whisper_paths {
            if path.exists() {
                self.whisper_cpp_path = Some(path.clone());
                break;
            }
        }
        
        // Look for Whisper model files
        let model_paths = [
            PathBuf::from("./models/whisper-base.safetensors"),
            PathBuf::from("./models/ggml-base.en.bin"),
            PathBuf::from("./models/whisper.cpp/ggml-base.en.bin"),
            PathBuf::from("./whisper.cpp/models/ggml-base.en.bin"),
        ];
        
        for path in &model_paths {
            if path.exists() {
                self.model_path = Some(path.clone());
                break;
            }
        }
        
        if self.whisper_cpp_path.is_none() {
            tracing::warn!("Whisper.cpp executable not found, using mock transcription");
        }
        
        if self.model_path.is_none() {
            tracing::warn!("Whisper model not found, using mock transcription");
        }
        
        Ok(())
    }
    
    /// Transcribe using whisper.cpp
    async fn transcribe_with_whisper_cpp(
        &self,
        audio_path: &PathBuf,
        whisper_path: &PathBuf,
        model_path: &PathBuf,
    ) -> Result<String> {
        let output = Command::new(whisper_path)
            .arg("-m")
            .arg(model_path)
            .arg("-f")
            .arg(audio_path)
            .arg("--output-txt")
            .arg("--no-timestamps")
            .output()
            .context("Failed to execute whisper.cpp")?;
        
        if output.status.success() {
            let transcription = String::from_utf8(output.stdout)
                .context("Invalid UTF-8 in whisper output")?;
            Ok(transcription.trim().to_string())
        } else {
            let error = String::from_utf8_lossy(&output.stderr);
            Err(anyhow!("Whisper transcription failed: {}", error).into())
        }
    }
    
    /// Mock transcription for development
    async fn mock_transcription(&self, audio_path: &PathBuf) -> Result<String> {
        // Simple mock based on file name and size
        let file_name = audio_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("audio");
        
        let file_size = fs::metadata(audio_path)
            .await
            .map(|m| m.len())
            .unwrap_or(0);
        
        // Generate mock transcription based on file characteristics
        let mock_content = if file_name.contains("meeting") {
            "This is a mock transcription of a meeting recording. The audio discussed project updates, action items, and next steps."
        } else if file_name.contains("call") {
            "This is a mock transcription of a phone call. The conversation covered business topics and follow-up actions."
        } else if file_name.contains("note") || file_name.contains("memo") {
            "This is a mock transcription of a voice note. The speaker shared thoughts and ideas on various topics."
        } else {
            "This is a mock transcription of an audio recording. The content would be transcribed here in a real implementation."
        };
        
        // Add length variation based on file size
        let extended_content = if file_size > 1024 * 1024 {
            format!("{} This appears to be a longer recording with additional details and extended discussion.", mock_content)
        } else {
            mock_content.to_string()
        };
        
        tracing::info!("Generated mock transcription for: {}", file_name);
        Ok(extended_content)
    }
    
    /// Check if Whisper is available and ready
    pub fn is_available(&self) -> bool {
        self.whisper_cpp_path.is_some() && self.model_path.is_some()
    }
    
    /// Get supported audio formats
    pub fn supported_formats() -> Vec<&'static str> {
        vec!["wav", "mp3", "m4a", "flac", "ogg"]
    }
}

// Legacy compatibility struct
pub struct Whisper;

impl Whisper {
    pub fn new() -> Result<Self> {
        Ok(Self)
    }
    
    pub async fn transcribe_audio(&self, _audio_data: &[u8]) -> Result<String> {
        Ok("Legacy Whisper transcription".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_whisper_processor_creation() {
        let processor = WhisperProcessor::new().await;
        assert!(processor.is_ok());
    }

    #[tokio::test]
    async fn test_mock_transcription() {
        let processor = WhisperProcessor::new().await.unwrap();
        
        // Create a temporary audio file
        let temp_dir = tempdir().unwrap();
        let audio_path = temp_dir.path().join("test_meeting.wav");
        let mut file = File::create(&audio_path).unwrap();
        file.write_all(b"fake audio data").unwrap();
        
        let result = processor.mock_transcription(&audio_path).await;
        assert!(result.is_ok());
        assert!(result.unwrap().contains("meeting"));
    }

    #[test]
    fn test_supported_formats() {
        let formats = WhisperProcessor::supported_formats();
        assert!(formats.contains(&"wav"));
        assert!(formats.contains(&"mp3"));
        assert!(formats.len() > 0);
    }
}
