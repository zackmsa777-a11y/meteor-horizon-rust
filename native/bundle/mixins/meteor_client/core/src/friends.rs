// SPDX-License-Identifier: GPL-3.0-only
// Native Friends/Friend model. Copyright (c) Meteor Development.
// Profile HTTP refresh and head textures are engine integrations, not performed here.
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Friend {
    pub name: String,
    /// Mojang UUID in undashed hexadecimal form; absent for unresolved/offline names.
    pub id: Option<String>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct Friends {
    pub friends: Vec<Friend>,
}

// Java's equalsIgnoreCase compares simple character case mappings, not expanding folds
// (so e.g. a sharp-s is not equal to two S characters). Minecraft profile names are ASCII.
fn simple_fold(c: char) -> char {
    fn upper(c: char) -> char {
        let mut chars = c.to_uppercase();
        let first = chars.next().unwrap_or(c);
        if chars.next().is_some() { c } else { first }
    }
    upper(c).to_lowercase().next().unwrap_or(c)
}
fn same_name(a: &str, b: &str) -> bool {
    a.chars().map(simple_fold).eq(b.chars().map(simple_fold))
}

impl Friends {
    /// Native command facade. Names can remain unresolved for offline play; the Java
    /// PlayerListEntry argument and online profile/head refresh are separate integrations.
    pub fn command(&mut self, args: &[&str]) -> Result<String, String> {
        match args {
            ["add", name] => {
                if self.add(Friend {
                    name: (*name).into(),
                    id: None,
                }) {
                    Ok(format!("Added {name} to friends."))
                } else {
                    Err("Already friends with that player, or invalid name.".into())
                }
            }
            ["remove", name] => {
                if self.remove(name) {
                    Ok(format!("Removed {name} from friends."))
                } else {
                    Err("Not friends with that player.".into())
                }
            }
            [] | ["list"] => Ok(format!(
                "Friends ({}): {}",
                self.friends.len(),
                self.friends
                    .iter()
                    .map(|f| f.name.as_str())
                    .collect::<Vec<_>>()
                    .join(", ")
            )),
            _ => Err("Usage: /meteor friends add NAME; remove NAME; list".into()),
        }
    }
    pub fn get(&self, name: &str) -> Option<&Friend> {
        self.friends
            .iter()
            .find(|friend| same_name(&friend.name, name))
    }

    /// Same name validation and insertion order as upstream Friends.add.
    pub fn add(&mut self, friend: Friend) -> bool {
        if friend.name.is_empty() || friend.name.contains(' ') || self.get(&friend.name).is_some() {
            return false;
        }
        self.friends.push(friend);
        true
    }

    /// Resolve the argument case-insensitively before removing the exact stored entry.
    pub fn remove(&mut self, name: &str) -> bool {
        let Some(index) = self.friends.iter().position(|f| same_name(&f.name, name)) else {
            return false;
        };
        self.friends.remove(index);
        true
    }

    pub fn should_attack(&self, name: Option<&str>) -> bool {
        name.is_none_or(|name| self.get(name).is_none())
    }

    /// Loading skips duplicate names and sorts; adding later retains insertion order.
    /// As in upstream fromTag, this intentionally does not apply add's name validation.
    pub fn loaded(entries: impl IntoIterator<Item = Friend>) -> Self {
        let mut result = Self::default();
        for friend in entries {
            if result.get(&friend.name).is_none() {
                result.friends.push(friend);
            }
        }
        result.friends.sort_by(|a, b| {
            a.name
                .chars()
                .map(simple_fold)
                .cmp(b.name.chars().map(simple_fold))
        });
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn friend(name: &str) -> Friend {
        Friend {
            name: name.into(),
            id: None,
        }
    }

    #[test]
    fn offline_names_and_case_insensitive_membership() {
        let mut friends = Friends::default();
        assert!(friends.add(friend("Zack")));
        assert!(!friends.add(friend("zAcK")));
        assert!(!friends.add(friend("")));
        assert!(!friends.add(friend("two names")));
        assert!(!friends.should_attack(Some("ZACK")));
        assert!(friends.should_attack(None));
        assert!(friends.should_attack(Some("Alex")));
        assert!(friends.remove("zack"));
        assert!(!friends.remove("Zack"));
    }

    #[test]
    fn load_deduplicates_sorts_and_retains_optional_identity() {
        let id = "0123456789abcdef0123456789abcdef";
        let entries = vec![
            friend("Zack"),
            Friend {
                name: "Alex".into(),
                id: Some(id.into()),
            },
            friend("ALEX"),
        ];
        let friends = Friends::loaded(entries);
        assert_eq!(friends.friends.len(), 2);
        assert_eq!(friends.friends[0].name, "Alex");
        let copy: Friends =
            serde_json::from_str(&serde_json::to_string(&friends).unwrap()).unwrap();
        assert_eq!(copy.get("alex").unwrap().id.as_deref(), Some(id));
        assert_eq!(copy.get("zack").unwrap().id, None);
    }

    #[test]
    fn simple_case_comparison_does_not_expand_characters() {
        assert!(same_name("İ", "i"));
        assert!(same_name("Σ", "ς"));
        assert!(!same_name("ß", "SS"));
    }

    #[test]
    fn commands_use_persistable_membership_and_report_rejections() {
        let mut friends = Friends::default();
        assert!(friends.command(&["add", "Alex"]).is_ok());
        assert!(friends.command(&["add", "ALEX"]).is_err());
        assert!(friends.command(&["list"]).unwrap().contains("Alex"));
        assert!(friends.command(&["remove", "alex"]).is_ok());
        assert!(friends.friends.is_empty());
        assert!(friends.command(&["remove", "alex"]).is_err());
    }
}
