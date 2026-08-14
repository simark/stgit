# Refuse git commands that would invalidate StGit's patch-stack metadata when
# the current branch has an StGit stack.  Source this file from your ~/.zshrc.
#
# This is a shell function rather than a script in PATH so that StGit itself
# (which shells out to `git reset --hard` for `stg rebase`/`stg pull`, among
# others) is unaffected.  Use `command git` to bypass it.
#
# The guard only bites when the branch actually has *applied* patches.  StGit
# records patch state only for applied patches: the stack base is recomputed as
# the branch head whenever the applied list is empty (Stack::from_branch), and
# check_head_top_mismatch() short-circuits to Ok in that case.  So with nothing
# applied -- e.g. right after `stg commit -a` -- the branch head is free to
# move, and StGit picks up the new base by itself.
git() {
  emulate -L zsh
  local -a dangerous=( commit rebase reset merge cherry-pick revert am )
  local -a gitopts=() rest=( "$@" )
  local sub

  # Skip git's global options to find the actual subcommand, keeping them so
  # the checks below inspect the same repository git itself would act on.
  while (( $#rest )); do
    case $rest[1] in
      -C|-c|--git-dir|--work-tree|--namespace|--exec-path|--config-env)
        gitopts+=( $rest[1] $rest[2] ); shift 2 rest ;;
      -*) gitopts+=( $rest[1] ); shift rest ;;
      *)  sub=$rest[1]; break ;;
    esac
  done

  [[ -n $sub ]] && (( dangerous[(Ie)$sub] )) || { command git "$@"; return }

  local branch=$(command git $gitopts symbolic-ref --quiet --short HEAD 2>/dev/null)
  [[ -n $branch ]] &&
    command git $gitopts rev-parse --verify --quiet "refs/stacks/$branch" >/dev/null ||
    { command git "$@"; return }

  # Forms of `git reset` that leave the branch ref alone: an explicit pathspec
  # (`--` rules out --soft/--hard, which git rejects with paths), or no
  # commit-ish at all (resets index/worktree to HEAD).  Neither can desync the
  # stack.  `git reset <arg>` without `--` stays blocked: <arg> may be a
  # commit-ish, and guessing wrong is the expensive direction.
  if [[ $sub == reset ]]; then
    local -a rargs=( $rest[2,-1] )
    local arg moves_head=0
    if (( ! rargs[(Ie)--] )); then
      for arg in $rargs; do
        [[ $arg == -* ]] || { moves_head=1; break }
      done
    fi
    (( moves_head )) || { command git "$@"; return }
  fi

  local toplevel=$(command git $gitopts rev-parse --show-toplevel 2>/dev/null)
  local applied
  if [[ -z $toplevel ]] ||
     ! applied=$(command stg -C $toplevel series --count --applied --branch $branch 2>/dev/null)
  then
    print -u2 "error: refusing 'git $sub' on StGit branch '$branch' (cannot read stack state)"
    print -u2 "use 'command git $sub ...' to override"
    return 1
  fi

  if (( applied )); then
    print -u2 "error: refusing 'git $sub' on StGit branch '$branch' ($applied patch(es) applied)"
    print -u2 "use the stg equivalent, or 'command git $sub ...' to override"
    return 1
  fi

  # No applied patches, so this is safe.  Unapplied/hidden patches survive and
  # stay reachable from refs/stacks/$branch, but they will be rebased onto
  # whatever the new base turns out to be the next time they are pushed.
  local total=$(command stg -C $toplevel series --count --all --branch $branch 2>/dev/null)
  if [[ -n $total ]] && (( total )); then
    print -u2 "warning: stack '$branch' has no applied patches, but $total unapplied/hidden remain"
    print -u2 "warning: they will be rebased onto the new base when pushed"
  fi

  command git "$@"
}
