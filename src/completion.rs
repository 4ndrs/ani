use clap::Command;
use clap_complete::{generate, shells};
use regex::RegexBuilder;

const ANIME_INFO_ZSH_COMPLETION: &str = r#"
_ani_info() {
    # a previous completion call already produced and displayed a list
    # reuse it and begin selecting the items
    if [[ "${compstate[old_list]}" == "shown" ]]; then
        compstate[old_list]=keep
        compstate[insert]=menu
        return
    fi

    local -a ids displays lines

    local line id title output

    local query="${words[CURRENT]}"

    if (( ${#query} < 2 )); then
        _message 'Type at least 2 characters to search AniList'
        return
    fi

    zle -M "Searching AniList for '$query'..."

    output="$(ani completion-search "$query" 2>/dev/null)"

    if [[ -z "$output" ]]; then
        _message "No anime found for '$query'"
        return
    fi

    lines=("${(f)output}")

    for line in "${lines[@]}"; do
        id="${line%%$'\t'*}"
        title="${line#*$'\t'}"

        ids+=("$id")
        displays+=("${(r:8:)id} $title")
    done

    compadd -U -Q -d displays -a ids

    # first tab shows the list but doesn't modify the command line
    compstate[list]='list force'
    compstate[insert]=''
}"#;

pub fn generate_completions(cmd: &mut Command) -> Result<String, Box<dyn std::error::Error>> {
    let mut buffer: Vec<u8> = Vec::new();

    let name = cmd.get_name().to_string();

    generate(shells::Zsh, cmd, name, &mut buffer);

    let info_function_regex = RegexBuilder::new(r"(?<start>\(info\).*?:id[^\n]*:)_default")
        .dot_matches_new_line(true)
        .multi_line(true)
        .build()?;

    let characters_function_regex =
        RegexBuilder::new(r"(?<start>\(characters\).*?:id[^\n]*:)_default")
            .dot_matches_new_line(true)
            .multi_line(true)
            .build()?;

    let zsh_completion = String::from_utf8(buffer)?;
    let zsh_completion = info_function_regex.replace(&zsh_completion, "${start}_ani_info");
    let zsh_completion = characters_function_regex.replace(&zsh_completion, "${start}_ani_info");

    let zsh_completion = zsh_completion.replacen(
        "#compdef ani",
        &format!("#compdef ani\n{ANIME_INFO_ZSH_COMPLETION}"),
        1,
    );

    Ok(zsh_completion)
}
