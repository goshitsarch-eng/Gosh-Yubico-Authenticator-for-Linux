import 'package:flutter/material.dart';
import 'package:simple_icons/simple_icons.dart';

/// Represents a service-specific icon with its brand color.
class ServiceIcon {
  final IconData icon;
  final Color color;

  const ServiceIcon(this.icon, this.color);
}

/// Maps issuer/account names to brand icons.
/// Returns null if no matching icon found.
ServiceIcon? getServiceIcon(String? issuer, String account) {
  final name = (issuer ?? account).toLowerCase().trim();

  // Check against known services
  for (final entry in serviceIconMap.entries) {
    for (final keyword in entry.key) {
      if (name.contains(keyword)) {
        return entry.value;
      }
    }
  }
  return null;
}

/// Get a service icon by its primary key (first keyword).
ServiceIcon? getServiceIconByKey(String key) {
  final keyLower = key.toLowerCase();
  for (final entry in serviceIconMap.entries) {
    if (entry.key.first == keyLower) {
      return entry.value;
    }
  }
  return null;
}

/// Get all available service icon keys for the icon picker.
List<String> getServiceIconKeys() {
  return serviceIconMap.keys.map((keywords) => keywords.first).toList();
}

/// Get a display name for a service key.
String getServiceDisplayName(String key) {
  // Capitalize the first letter of each word
  return key.split(' ').map((word) {
    if (word.isEmpty) return word;
    // Handle special cases like "1password"
    if (word == '1password') return '1Password';
    if (word == 'aws') return 'AWS';
    if (word == 'gcp') return 'GCP';
    if (word == 'npm') return 'npm';
    if (word == 'openai') return 'OpenAI';
    return word[0].toUpperCase() + word.substring(1);
  }).join(' ');
}

/// Mapping of keywords to service icons (exported for icon picker).
/// Gmail must come before Google so it gets its own dedicated icon.
const serviceIconMap = <List<String>, ServiceIcon>{
  // Email services (Gmail before Google)
  ['gmail']: ServiceIcon(SimpleIcons.gmail, SimpleIconColors.gmail),

  // Cloud providers
  ['aws', 'amazon web services']: ServiceIcon(SimpleIcons.amazonaws, SimpleIconColors.amazonaws),
  ['google', 'gcp']: ServiceIcon(SimpleIcons.google, SimpleIconColors.google),
  ['microsoft', 'azure', 'outlook', 'office365']: ServiceIcon(SimpleIcons.microsoft, SimpleIconColors.microsoft),
  ['cloudflare']: ServiceIcon(SimpleIcons.cloudflare, SimpleIconColors.cloudflare),
  ['digitalocean']: ServiceIcon(SimpleIcons.digitalocean, SimpleIconColors.digitalocean),

  // Development
  ['github']: ServiceIcon(SimpleIcons.github, SimpleIconColors.github),
  ['gitlab']: ServiceIcon(SimpleIcons.gitlab, SimpleIconColors.gitlab),
  ['bitbucket']: ServiceIcon(SimpleIcons.bitbucket, SimpleIconColors.bitbucket),
  ['docker']: ServiceIcon(SimpleIcons.docker, SimpleIconColors.docker),
  ['npm']: ServiceIcon(SimpleIcons.npm, SimpleIconColors.npm),

  // AI/ML
  ['openai', 'chatgpt']: ServiceIcon(SimpleIcons.openai, SimpleIconColors.openai),

  // Social/Communication
  ['discord']: ServiceIcon(SimpleIcons.discord, SimpleIconColors.discord),
  ['slack']: ServiceIcon(SimpleIcons.slack, SimpleIconColors.slack),
  ['twitter', 'x.com']: ServiceIcon(SimpleIcons.x, SimpleIconColors.x),
  ['facebook', 'meta']: ServiceIcon(SimpleIcons.facebook, SimpleIconColors.facebook),
  ['linkedin']: ServiceIcon(SimpleIcons.linkedin, SimpleIconColors.linkedin),
  ['reddit']: ServiceIcon(SimpleIcons.reddit, SimpleIconColors.reddit),

  // Automation
  ['zapier']: ServiceIcon(SimpleIcons.zapier, SimpleIconColors.zapier),

  // Finance
  ['paypal']: ServiceIcon(SimpleIcons.paypal, SimpleIconColors.paypal),
  ['stripe']: ServiceIcon(SimpleIcons.stripe, SimpleIconColors.stripe),

  // Gaming
  ['steam']: ServiceIcon(SimpleIcons.steam, SimpleIconColors.steam),
  ['twitch']: ServiceIcon(SimpleIcons.twitch, SimpleIconColors.twitch),
  ['epic', 'epicgames']: ServiceIcon(SimpleIcons.epicgames, SimpleIconColors.epicgames),

  // Password managers
  ['1password', 'onepassword']: ServiceIcon(SimpleIcons.n1password, SimpleIconColors.n1password),
  ['bitwarden']: ServiceIcon(SimpleIcons.bitwarden, SimpleIconColors.bitwarden),

  // Other popular
  ['dropbox']: ServiceIcon(SimpleIcons.dropbox, SimpleIconColors.dropbox),
  ['apple', 'icloud']: ServiceIcon(SimpleIcons.apple, SimpleIconColors.apple),
  ['spotify']: ServiceIcon(SimpleIcons.spotify, SimpleIconColors.spotify),
  ['netflix']: ServiceIcon(SimpleIcons.netflix, SimpleIconColors.netflix),
  ['proton', 'protonmail']: ServiceIcon(SimpleIcons.proton, SimpleIconColors.proton),
};
