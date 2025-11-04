use std::{fs, path::Path};

pub fn parse_general(
    p: &Path,
) -> anyhow::Result<(Option<String>, Option<String>, Option<i64>, Option<i64>)> {
    if !p.exists() {
        return Ok((None, None, None, None));
    }
    let text = fs::read_to_string(p)?;
    let mut in_general = false;
    let mut name = None;
    let mut icon_key = None;
    let mut last = None;
    let mut total = None;

    for raw in text.lines() {
        let line = raw.trim();
        if line.starts_with('[') && line.ends_with(']') {
            in_general = &line[1..line.len() - 1] == "General";
            continue;
        }
        if !in_general || line.is_empty() {
            continue;
        }

        // strip unescaped '#'
        let mut s = line;
        if let Some(idx) = s
            .bytes()
            .enumerate()
            .find(|&(i, b)| b == b'#' && (i == 0 || s.as_bytes()[i - 1] != b'\\'))
            .map(|(i, _)| i)
        {
            s = &s[..idx].trim();
        }
        if s.is_empty() {
            continue;
        }

        let Some(eq) = s.find('=') else {
            continue;
        };
        let key = s[..eq].trim();
        let mut val = s[eq + 1..].trim().to_string();

        val = prism_unquote(prism_unescape(&val));

        match key {
            "name" => name = Some(val),
            "iconKey" => icon_key = Some(val),
            "lastLaunchTime" => {
                if let Ok(v) = val.parse() {
                    last = Some(v);
                }
            }
            "totalTimePlayed" => {
                if let Ok(v) = val.parse() {
                    total = Some(v);
                }
            }
            _ => {}
        }
    }
    Ok((name, icon_key, last, total))
}

fn prism_unescape(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut prev_bs = false;
    for ch in s.chars() {
        if prev_bs {
            match ch {
                'n' => out.push('\n'),
                't' => out.push('\t'),
                '#' => out.push('#'),
                _ => out.push(ch),
            }
            prev_bs = false;
        } else if ch == '\\' {
            prev_bs = true;
        } else {
            out.push(ch);
        }
    }
    if prev_bs {
        out.push('\\');
    }
    out
}

fn prism_unquote(mut s: String) -> String {
    let has_special = s.contains(';') || s.contains('=') || s.contains(',');
    if has_special && s.len() >= 2 && s.starts_with('"') && s.ends_with('"') {
        s.remove(0);
        s.pop();
    }
    s
}
