#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub(crate) enum CommandKind {
    UpdateMajor,
    UpdateMinor,
    UpdatePatch,
    Skip,
    Diff,
    Quit,
    Help,
}

struct CommandMeta {
    pub kind: CommandKind,
    pub key: char,
    pub help: &'static str,
}

const COMMAND_LIST: [CommandMeta; 7] = [
    CommandMeta {
        kind: CommandKind::UpdateMajor,
        key: '1',
        help: "update package major version",
    },
    CommandMeta {
        kind: CommandKind::UpdateMinor,
        key: '2',
        help: "update package minor version",
    },
    CommandMeta {
        kind: CommandKind::UpdatePatch,
        key: '3',
        help: "update package patch version",
    },
    CommandMeta {
        kind: CommandKind::Skip,
        key: 's',
        help: "skip current package",
    },
    CommandMeta {
        kind: CommandKind::Diff,
        key: 'd',
        help: "show diff for current package",
    },
    CommandMeta {
        kind: CommandKind::Quit,
        key: 'q',
        help: "quit; do not update package or any of the remaining ones",
    },
    CommandMeta {
        kind: CommandKind::Help,
        key: '?',
        help: "print help",
    },
];

pub(crate) fn get_command_kind_from_input(input: &str) -> Option<CommandKind> {
    let mut chars = input.chars();
    let key = chars.next()?;
    if chars.next().is_some() {
        return None;
    }
    COMMAND_LIST
        .iter()
        .find(|&command| command.key == key)
        .map(|command| command.kind)
}

pub(crate) fn get_command_key_text() -> String {
    COMMAND_LIST
        .iter()
        .map(|command| command.key.to_string())
        .collect::<Vec<String>>()
        .join(",")
}

pub(crate) fn get_command_help() -> String {
    COMMAND_LIST
        .iter()
        .map(|command| format!("{} - {}", command.key, command.help))
        .collect::<Vec<String>>()
        .join("\n")
}
