use std::fs::File;
use std::io::BufReader;
use std::path::PathBuf;

/// Map event types to Windows system sound file names
fn get_system_sound_name(event_type: &str) -> Option<&'static str> {
    match event_type {
        "complete" => Some("Windows Notify System Generic.wav"),
        "error" => Some("Windows Critical Stop.wav"),
        "attention" => Some("Windows Notify Calendar.wav"),
        "stop" => Some("Windows Notify Email.wav"),
        "session-end" => Some("Windows Logoff Sound.wav"),
        _ => None,
    }
}

/// Alternative sound file names for different Windows versions
fn get_alternative_sound_names(event_type: &str) -> &'static [&'static str] {
    match event_type {
        "complete" => &["notify.wav", "tada.wav", "Windows Notify.wav"],
        "error" => &["Windows Error.wav", "Windows Hardware Fail.wav", "chord.wav"],
        "attention" => &["Windows Notify.wav", "Windows Balloon.wav", "ding.wav"],
        "stop" => &["Windows Information Bar.wav", "Windows Unlock.wav"],
        "session-end" => &["Windows Shutdown.wav", "Windows Logoff.wav"],
        _ => &[],
    }
}

/// Get the full path to a sound file, checking system dir then alternatives.
/// Returns None if no sound file is found (e.g., on Linux dev machines).
pub fn get_sound_path(event_type: &str) -> Option<PathBuf> {
    let sound_name = get_system_sound_name(event_type)?;

    // Try Windows system sounds directory
    let system_path = PathBuf::from(r"C:\Windows\Media").join(sound_name);
    if system_path.exists() {
        return Some(system_path);
    }

    // Try alternative Windows sound names (some versions differ)
    for alt_name in get_alternative_sound_names(event_type) {
        let alt_path = PathBuf::from(r"C:\Windows\Media").join(alt_name);
        if alt_path.exists() {
            return Some(alt_path);
        }
    }

    // No sound file found — graceful fallback (expected on Linux dev)
    log::warn!(
        "Sound file '{}' not found for event '{}' (expected on non-Windows)",
        sound_name,
        event_type
    );
    None
}

/// Play a sound for the given event type (non-blocking).
/// Spawns a thread for playback so the caller returns immediately.
/// Gracefully handles missing sound files and audio device errors.
pub fn play_event_sound(event_type: &str) {
    let path = match get_sound_path(event_type) {
        Some(p) => p,
        None => {
            log::debug!("No sound file available for event: {}", event_type);
            return;
        }
    };

    log::info!("Playing sound for '{}': {:?}", event_type, path);

    // Spawn a thread for non-blocking playback
    std::thread::spawn(move || {
        if let Err(e) = play_wav_file(&path) {
            log::error!("Failed to play sound {:?}: {}", path, e);
        }
    });
}

/// Play a .wav file using rodio.
/// The OutputStream and Sink must stay alive until playback finishes.
fn play_wav_file(path: &PathBuf) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let (_stream, stream_handle) = rodio::OutputStream::try_default()?;
    let file = File::open(path)?;
    let source = rodio::Decoder::new(BufReader::new(file))?;
    let sink = rodio::Sink::try_new(&stream_handle)?;
    sink.append(source);
    sink.sleep_until_end();
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sound_mapping_all_events() {
        let events = ["complete", "error", "attention", "stop", "session-end"];
        for event in &events {
            let name = get_system_sound_name(event);
            assert!(
                name.is_some(),
                "Event '{}' should have a sound mapping",
                event
            );
        }
    }

    #[test]
    fn test_sound_mapping_unknown_event() {
        assert!(get_system_sound_name("unknown").is_none());
        assert!(get_system_sound_name("").is_none());
        assert!(get_system_sound_name("hack").is_none());
    }

    #[test]
    fn test_sound_file_names_are_wav() {
        let events = ["complete", "error", "attention", "stop", "session-end"];
        for event in &events {
            let name = get_system_sound_name(event).unwrap();
            assert!(
                name.ends_with(".wav"),
                "Sound for '{}' should be a .wav file, got '{}'",
                event,
                name
            );
        }
    }

    #[test]
    fn test_alternative_sound_names_exist() {
        // All event types should have alternative sound names
        let events = ["complete", "error", "attention", "stop", "session-end"];
        for event in &events {
            let alts = get_alternative_sound_names(event);
            assert!(
                !alts.is_empty(),
                "Event '{}' should have alternative sound names",
                event
            );
        }
    }

    #[test]
    fn test_alternative_sound_names_unknown_event() {
        let alts = get_alternative_sound_names("unknown");
        assert!(alts.is_empty());
    }

    #[test]
    fn test_get_sound_path_graceful_on_linux() {
        // On Linux, C:\Windows\Media doesn't exist, so get_sound_path
        // should return None gracefully (not panic)
        let result = get_sound_path("complete");
        // We don't assert the value — it depends on the OS.
        // The key assertion is that this doesn't panic.
        if cfg!(not(target_os = "windows")) {
            assert!(
                result.is_none(),
                "On non-Windows, sound path should be None"
            );
        }
    }

    #[test]
    fn test_get_sound_path_unknown_event_returns_none() {
        assert!(get_sound_path("unknown").is_none());
    }

    #[test]
    fn test_play_event_sound_unknown_event_no_panic() {
        // Should not panic even for unknown events
        play_event_sound("unknown");
    }

    #[test]
    fn test_play_event_sound_graceful_on_linux() {
        // On Linux, sound files won't exist — should not panic
        play_event_sound("complete");
    }

    #[test]
    fn test_each_event_maps_to_distinct_sound() {
        let events = ["complete", "error", "attention", "stop", "session-end"];
        let names: Vec<&str> = events
            .iter()
            .map(|e| get_system_sound_name(e).unwrap())
            .collect();

        // Verify all sounds are distinct
        for (i, name) in names.iter().enumerate() {
            for (j, other) in names.iter().enumerate() {
                if i != j {
                    assert_ne!(
                        name, other,
                        "Events '{}' and '{}' should have distinct sounds",
                        events[i], events[j]
                    );
                }
            }
        }
    }
}
