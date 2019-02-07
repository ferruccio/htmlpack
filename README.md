# htmlpack

The purpose of `htmlpack` is to take an html document and embed any supporting
files, such as images, directly into the html file itself. It does this by
replacing URIs which refer to files with data URIs.

Currently, it only supports updating `src` attributes in `img` tags.
