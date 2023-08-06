# templates

Hello potentially template developing friend! I'm valkyrie, and today we'll be learning how to make a simple change to a template.

At the top of every template is its input struct. it looks like this:

```jinja2
    {#
    struct Input {
        cdn_url: string, // documentation
        root_url: string, // more documentation
        logged_in_user: User?
    }
    struct User {
        id: int,
        name: string
    }
    #}
```

this defines the tree of variables that can be referenced within tempates, like so

```jinja2
{{ variable }}
{{ struct.variable }}
```
