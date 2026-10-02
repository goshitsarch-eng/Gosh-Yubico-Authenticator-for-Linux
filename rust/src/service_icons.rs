/// Brand color plus a short label used for native avatars.
#[derive(Debug, Clone, Copy)]
pub struct ServiceIcon {
    pub key: &'static str,
    pub label: &'static str,
    pub color: (u8, u8, u8),
}

const SERVICES: &[ServiceIcon] = &[
    ServiceIcon {
        key: "gmail",
        label: "Gmail",
        color: (0xEA, 0x43, 0x35),
    },
    ServiceIcon {
        key: "aws",
        label: "AWS",
        color: (0xFF, 0x99, 0x00),
    },
    ServiceIcon {
        key: "google",
        label: "Google",
        color: (0x42, 0x85, 0xF4),
    },
    ServiceIcon {
        key: "microsoft",
        label: "Microsoft",
        color: (0x00, 0xA4, 0xEF),
    },
    ServiceIcon {
        key: "cloudflare",
        label: "Cloudflare",
        color: (0xF3, 0x80, 0x20),
    },
    ServiceIcon {
        key: "digitalocean",
        label: "DigitalOcean",
        color: (0x00, 0x84, 0xFF),
    },
    ServiceIcon {
        key: "github",
        label: "GitHub",
        color: (0x18, 0x17, 0x17),
    },
    ServiceIcon {
        key: "gitlab",
        label: "GitLab",
        color: (0xFC, 0x6D, 0x26),
    },
    ServiceIcon {
        key: "bitbucket",
        label: "Bitbucket",
        color: (0x00, 0x52, 0xCC),
    },
    ServiceIcon {
        key: "docker",
        label: "Docker",
        color: (0x24, 0x96, 0xED),
    },
    ServiceIcon {
        key: "npm",
        label: "npm",
        color: (0xCB, 0x38, 0x37),
    },
    ServiceIcon {
        key: "openai",
        label: "OpenAI",
        color: (0x41, 0x2A, 0x4C),
    },
    ServiceIcon {
        key: "discord",
        label: "Discord",
        color: (0x58, 0x65, 0xF2),
    },
    ServiceIcon {
        key: "slack",
        label: "Slack",
        color: (0x4A, 0x15, 0x4B),
    },
    ServiceIcon {
        key: "twitter",
        label: "X",
        color: (0x00, 0x00, 0x00),
    },
    ServiceIcon {
        key: "facebook",
        label: "Facebook",
        color: (0x18, 0x77, 0xF2),
    },
    ServiceIcon {
        key: "linkedin",
        label: "LinkedIn",
        color: (0x0A, 0x66, 0xC2),
    },
    ServiceIcon {
        key: "reddit",
        label: "Reddit",
        color: (0xFF, 0x45, 0x00),
    },
    ServiceIcon {
        key: "zapier",
        label: "Zapier",
        color: (0xFF, 0x4A, 0x00),
    },
    ServiceIcon {
        key: "paypal",
        label: "PayPal",
        color: (0x00, 0x30, 0x87),
    },
    ServiceIcon {
        key: "stripe",
        label: "Stripe",
        color: (0x63, 0x5B, 0xFF),
    },
    ServiceIcon {
        key: "steam",
        label: "Steam",
        color: (0x17, 0x1A, 0x21),
    },
    ServiceIcon {
        key: "twitch",
        label: "Twitch",
        color: (0x91, 0x46, 0xFF),
    },
    ServiceIcon {
        key: "epic",
        label: "Epic Games",
        color: (0x31, 0x31, 0x31),
    },
    ServiceIcon {
        key: "1password",
        label: "1Password",
        color: (0x00, 0x9B, 0xDE),
    },
    ServiceIcon {
        key: "bitwarden",
        label: "Bitwarden",
        color: (0x17, 0x50, 0xDD),
    },
    ServiceIcon {
        key: "dropbox",
        label: "Dropbox",
        color: (0x00, 0x61, 0xFF),
    },
    ServiceIcon {
        key: "apple",
        label: "Apple",
        color: (0x55, 0x55, 0x55),
    },
    ServiceIcon {
        key: "spotify",
        label: "Spotify",
        color: (0x1D, 0xB9, 0x54),
    },
    ServiceIcon {
        key: "netflix",
        label: "Netflix",
        color: (0xE5, 0x09, 0x14),
    },
    ServiceIcon {
        key: "proton",
        label: "Proton",
        color: (0x6D, 0x4A, 0xFF),
    },
];

const KEYWORDS: &[(&[&str], &str)] = &[
    (&["gmail"], "gmail"),
    (&["aws", "amazon web services"], "aws"),
    (&["google", "gcp"], "google"),
    (&["microsoft", "azure", "outlook", "office365"], "microsoft"),
    (&["cloudflare"], "cloudflare"),
    (&["digitalocean"], "digitalocean"),
    (&["github"], "github"),
    (&["gitlab"], "gitlab"),
    (&["bitbucket"], "bitbucket"),
    (&["docker"], "docker"),
    (&["npm"], "npm"),
    (&["openai", "chatgpt"], "openai"),
    (&["discord"], "discord"),
    (&["slack"], "slack"),
    (&["twitter", "x.com"], "twitter"),
    (&["facebook", "meta"], "facebook"),
    (&["linkedin"], "linkedin"),
    (&["reddit"], "reddit"),
    (&["zapier"], "zapier"),
    (&["paypal"], "paypal"),
    (&["stripe"], "stripe"),
    (&["steam"], "steam"),
    (&["twitch"], "twitch"),
    (&["epic", "epicgames"], "epic"),
    (&["1password", "onepassword"], "1password"),
    (&["bitwarden"], "bitwarden"),
    (&["dropbox"], "dropbox"),
    (&["apple", "icloud"], "apple"),
    (&["spotify"], "spotify"),
    (&["netflix"], "netflix"),
    (&["proton", "protonmail"], "proton"),
];

pub fn all_services() -> &'static [ServiceIcon] {
    SERVICES
}

pub fn get_service_by_key(key: &str) -> Option<&'static ServiceIcon> {
    SERVICES.iter().find(|s| s.key.eq_ignore_ascii_case(key))
}

pub fn guess_service(issuer: Option<&str>, account: &str) -> Option<&'static ServiceIcon> {
    let name = issuer.unwrap_or(account).to_ascii_lowercase();
    for (keywords, key) in KEYWORDS {
        if keywords.iter().any(|keyword| name.contains(keyword)) {
            return get_service_by_key(key);
        }
    }
    None
}

/// Best-effort domain for a credential, used for favicon lookup.
pub fn guess_domain(issuer: Option<&str>, account: &str) -> Option<String> {
    if let Some(at) = account.find('@') {
        let domain = account[at + 1..].trim();
        if domain.contains('.') && !domain.is_empty() {
            return Some(domain.to_ascii_lowercase());
        }
    }
    issuer.and_then(|issuer| {
        let cleaned = issuer.trim().to_ascii_lowercase();
        if cleaned.contains('.') {
            Some(cleaned)
        } else {
            None
        }
    })
}

pub fn extract_domain_from_text(text: &str) -> Option<String> {
    let cleaned = text.trim().to_ascii_lowercase();
    if cleaned.contains('.') && !cleaned.contains(' ') {
        Some(
            cleaned
                .trim_start_matches("https://")
                .trim_start_matches("http://")
                .trim_end_matches('/')
                .to_string(),
        )
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gmail_before_google() {
        let icon = guess_service(Some("Gmail"), "user@example.com").unwrap();
        assert_eq!(icon.key, "gmail");
    }

    #[test]
    fn github_match() {
        let icon = guess_service(Some("GitHub"), "octocat").unwrap();
        assert_eq!(icon.key, "github");
    }
}
