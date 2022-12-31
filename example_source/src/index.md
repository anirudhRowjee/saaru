---
title: what can this blog do?
description: just a small feature demonstration of the capabilites of this blog
wip: true
template: hello2.jinja
tags:
  - look
  - ma
  - i
  - can
  - use
  - tags
---

# Markdown Test Blog Post

This is a test markdown file.

It *should* _parse_ **soon**.

```python
print("Hello, world!")
```


# An h1 header

Paragraphs are separated by a blank line.

2nd paragraph. _Italic_, **bold**, and `monospace`. Itemized lists
look like:

- this one
- that one
- the other one

Note that --- not considering the asterisk --- the actual text
content starts at 4-columns in.

> Block quotes are
> written like so.
>
> They can span multiple paragraphs,
> if you like.

Use 3 dashes for an em-dash. Use 2 dashes for ranges (ex., "it's all
in chapters 12--14"). Three dots ... will be converted to an ellipsis.
Unicode is supported. ☺

## An h2 header

Here's a numbered list:

1.  first item
2.  second item
3.  third item

Note again how the actual text starts at 4 columns in (4 characters
from the left side). Here's a code sample:

```
# Let me re-iterate ...
for i in 1 .. 10 { do-something(i) }
```

As you probably guessed, indented 4 spaces. By the way, instead of
indenting the block, you can use delimited blocks, if you like:

```
define foobar() {
    print "Welcome to flavor country!";
}
```

NOW FOR SYNTAX HIGHLIGHTING!

```python
import time
# Quick, count to ten!
for i in range(10):
    # (but not *too* quick)
    time.sleep(0.5)
    print(i)
```

some more python highlighting

```py:index.py
@requires_authorization
def somefunc(param1='', param2=0):
    r'''A docstring'''
    if param1 > param2: # interesting
        print 'Gre\'ater'
    return (param2 - param1 + 1 + 0b10l) or None

class SomeClass:
    pass

>>> message = '''interpreter
... prompt'''
```

Here is some text which is large

```js
module.exports = {
  dest: 'docs',
  title: 'Hello VuePress World',
  description: 'VuePress install config',
  head: [
    ['link', { rel: 'icon', href: `/logo.png` }],
    ['link', { rel: 'manifest', href: '/manifest.json' }],
    ['meta', { name: 'theme-color', content: '#3eaf7c' }],
    ['meta', { name: 'apple-mobile-web-app-capable', content: 'yes' }],
    ['meta', { name: 'apple-mobile-web-app-status-bar-style', content: 'black' }],
    ['link', { rel: 'apple-touch-icon', href: `/icons/apple-touch-icon-152x152.png` }],
    ['link', { rel: 'mask-icon', href: '/icons/safari-pinned-tab.svg', color: '#3eaf7c' }],
    ['meta', { name: 'msapplication-TileImage', content: '/icons/msapplication-icon-144x144.png' }],
    ['meta', { name: 'msapplication-TileColor', content: '#000000' }]
  ],
  serviceWorker: true,
  ga: 'UA-109510157-1',
```

### An h3 header

Now a nested list:

1.  First, get these ingredients:

    - carrots
    - celery
    - lentils

2.  Boil some water.

3.  Dump everything in the pot and follow
    this algorithm:

        find wooden spoon
        uncover pot
        stir
        cover pot
        balance wooden spoon precariously on pot handle
        wait 10 minutes
        goto first step (or shut off burner when done)

    Do not bump wooden spoon or it will fall.

Notice again how text always lines up on 4-space indents (including
that last line which continues item 3 above).

Here's a link to [a website](http://foo.bar), to a [local
doc](local-doc.html), and to a [section heading in the current
doc](#an-h2-header)

What about footnotes?[^1]

Tables can look like this:

| Name         | Size | Material    | Color       |
| ------------ | ---- | ----------- | ----------- |
| All Business | 9    | leather     | brown       |
| Roundabout   | 10   | hemp canvas | natural     |
| Cinderella   | 11   | glass       | transparent |

Table: Shoes sizes, materials, and colors.

A horizontal rule follows.

---

And images can be specified like so:

![example image "this is nice"](https://external-content.duckduckgo.com/iu/?u=https%3A%2F%2Ftse1.mm.bing.net%2Fth%3Fid%3DOIP.XBuTOZYvrMgYELZ9HzJNZQHaEK%26pid%3DApi&f=1 'An exemplary image')

Inline math equation: $\omega = d\phi / dt$. Display
math should get its own line like so:

$$I = \int \rho R^{2} dV$$

And note that you can backslash-escape any punctuation characters
which you wish to be displayed literally, ex.: \`foo\`, \*bar\*, etc.

What if I want to embed a video?

Nice, isn't it?

Here's to some good writing! :champagne: I'll see y'all around.

==Adios!==

<div class='adm adm_info'>
    <p class="title"> **ℹ️  INFO** This is a  info admonition </p>
    <div class="body">
    here is where I'll show you what you need to know.

    I can even add a code block in here!

    ```rust
    println!("This is nice.");
    ```
    </div>

</div>

<div class='adm adm_warning'>
    <p class="title"> **⚠️  WARNING** This is a  warning admonition </p>
    <div class="body">
    nice, **this** is nice
    </div>
</div>

<div class='adm adm_note'>
    <p class="title">**✍️  NOTE** This is a  note admonition </p>
    <div class="body">
    Here is where i'll tell you to take note of something.
    </div>
</div>

<div class='adm adm_remember'>
    <p class="title"> **🧠 REMEMBER** This is for you to remember something.</p>
    <div class="body">
    nice, **this** is nice
    </div>
</div>

<div class='adm adm_bug'>
    <p class="title"> **🐞 BUG** Here's a bug! </p>
    <div class="body">
    The issue's on line 10 of this code.

    ```c
    #include <stdio.h>
    int main() {
        printf("hello, world!\n");
    }
    ```
</div>


[^1]: footnotes are fun!
