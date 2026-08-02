pub const ANIME_INFO_ZSH_COMPLETION: &str = r#"_ani_info() {
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
}
"#;
