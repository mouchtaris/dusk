--- vim: et ts=4 sw=4
## Install ci in your build images

If you are thinking about installing `ci` in your build images,
to reuse the same recipes, tooling, and general goodness in the
rest of your pipeline, you are thinking it right.

## Generic target form build recipes
```
add_target base;
    package curl git gcc cargo sudo;
    foreach ["bob"] { |user|
        user -D -G docker -i 1000 $user;
        sudo_user_nopsw = $user
    };

def binary = print0
    "xs2"
    "xs-run"
    "xs-compile"
    "xs-debug"
;

add_target build;
    untar_from_ctx release.tar /var/release;
    run xs2 /var/release/ci/ci-install-system;

add_target dist;
    foreach $binary {
        let bin = $argv[1];
        copy_from_target build
            /var/release/bin/$bin
            /usr/local/bin/$bin
    };
```

* Run with `-imakefile` to load `makefile.dust` as hooks for `add_target` etc.
* Run with `-i [gitlab, dockerfile, earthfile, makefile]`
* Run with `-i[debian,archlinux,ubuntu,gentoo]` as hooks for `package`, etc.
* Run with `-isudo` as generic hooks such as `sudo_*`.
