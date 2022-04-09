# Tutorial of dusk

## Invocations

In dusk everything revolves around invocations.

There are two main types of invocations:

* System commands
* Dusk methods

However, there are no differences in the way the are constructed.

### General invocation syntax

The general invocation layout, for any kind of invocation is:

    target_command_name

        @./optionally/at/some/other/cwd

        < $optionally_redirect_input_from_some_binding

        OPTIONAL    = environment_settings
        ENVIRONMENT = "setting to some value"
        another_env = "lower-case, whatever that means"

        and the rest "are arguments"
    ;

Notice how whitespace is non-important.

However, the order of those specifications *is*, and therefore they
cannot be reordered. Optional ones can be omitted.

This syntax is used for both dusk methods and system commands.

*However*, all settings (such as cwd, environment and redirections) are
currently *ignored* for method calls. A warning should be displayed
if they are used inappropriately [`issue-1`].

## Bindings

** needs: Jobs, Collection **

The other major point that things revolve around in dusk are bindings.

There are three types of bindings (currently).

* `let` bindings
* `def` bindings    (to be renamed to `fn`)
* `src` bindings

What differentiates them are two things:

* When is the job started
* When is the job collected

Those differencies can be summarized as follows:

| Binding   | Start it      | Collect it    |
| --------- | ------------- | ------------- |
| `let`     | now           | now           |
| `src`     | now           | later         |
| `def`     | later         | later         |

[`issue-1`]: Display warnings when redirections, environment or cwd are used in method calls.

<!-- vim: et ts=4 sw=4
-->
