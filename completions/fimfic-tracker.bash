_fimfic-tracker() {
    local i cur prev opts cmd
    COMPREPLY=()
    cur="${COMP_WORDS[COMP_CWORD]}"
    prev="${COMP_WORDS[COMP_CWORD-1]}"
    cmd=""
    opts=""

    for i in ${COMP_WORDS[@]}
    do
        case "${cmd},${i}" in
            ",$1")
                cmd="fimfic__tracker"
                ;;
            fimfic__tracker,d)
                cmd="fimfic__tracker__download"
                ;;
            fimfic__tracker,download)
                cmd="fimfic__tracker__download"
                ;;
            fimfic__tracker,help)
                cmd="fimfic__tracker__help"
                ;;
            fimfic__tracker,l)
                cmd="fimfic__tracker__list"
                ;;
            fimfic__tracker,list)
                cmd="fimfic__tracker__list"
                ;;
            fimfic__tracker,ls)
                cmd="fimfic__tracker__list"
                ;;
            fimfic__tracker,t)
                cmd="fimfic__tracker__track"
                ;;
            fimfic__tracker,track)
                cmd="fimfic__tracker__track"
                ;;
            fimfic__tracker,u)
                cmd="fimfic__tracker__untrack"
                ;;
            fimfic__tracker,untrack)
                cmd="fimfic__tracker__untrack"
                ;;
            fimfic__tracker__help,download)
                cmd="fimfic__tracker__help__download"
                ;;
            fimfic__tracker__help,help)
                cmd="fimfic__tracker__help__help"
                ;;
            fimfic__tracker__help,list)
                cmd="fimfic__tracker__help__list"
                ;;
            fimfic__tracker__help,track)
                cmd="fimfic__tracker__help__track"
                ;;
            fimfic__tracker__help,untrack)
                cmd="fimfic__tracker__help__untrack"
                ;;
            *)
                ;;
        esac
    done

    case "${cmd}" in
        fimfic__tracker)
            opts="-c -v -h -V --config --verbose --color --help --version track untrack list download help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 1 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                --config)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                -c)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --color)
                    COMPREPLY=($(compgen -W "auto always never" -- "${cur}"))
                    return 0
                    ;;
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        fimfic__tracker__download)
            opts="-f -y -n -h --force --yes --no --help [ID_OR_URL]..."
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 2 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        fimfic__tracker__help)
            opts="track untrack list download help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 2 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        fimfic__tracker__help__download)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        fimfic__tracker__help__help)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        fimfic__tracker__help__list)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        fimfic__tracker__help__track)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        fimfic__tracker__help__untrack)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        fimfic__tracker__list)
            opts="-s -r -h --short --sort-by --reverse --complete --show-complete --incomplete --show-incomplete --hiatus --show-hiatus --cancelled --show-cancelled --help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 2 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                --sort-by)
                    COMPREPLY=($(compgen -W "id title author chapters words update" -- "${cur}"))
                    return 0
                    ;;
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        fimfic__tracker__track)
            opts="-o -s -h --overwrite --skip-download --help <ID_OR_URL>..."
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 2 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        fimfic__tracker__untrack)
            opts="-h --help <ID_OR_URL>..."
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 2 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
    esac
}

complete -F _fimfic-tracker -o bashdefault -o default fimfic-tracker
