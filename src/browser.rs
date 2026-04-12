use std::process::Command;

/// Open a list of URLs in the default browser using `open` command.
pub fn open(urls: Option<&Vec<String>>) -> Result<(), String> {
    let Some(urls) = urls.filter(|u| !u.is_empty()) else {
        return Ok(());
    };

    for url in urls {
        Command::new("open")
            .arg(url)
            .spawn()
            .map_err(|e| format!("Failed to open URL '{url}': {e}"))?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn none_urls_is_ok() {
        assert!(open(None).is_ok());
    }

    #[test]
    fn empty_urls_is_ok() {
        let urls = vec![];
        assert!(open(Some(&urls)).is_ok());
    }
}
