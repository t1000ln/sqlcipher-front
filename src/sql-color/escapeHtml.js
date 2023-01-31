/*
 * Simplified version of the escape-html library which can be found at
 * https://github.com/component/escape-html
 *
 * Original license:
 * (The MIT License)
 *
 * Copyright (c) 2012-2013 TJ Holowaychuk
 * Copyright (c) 2015 Andreas Lubbe
 * Copyright (c) 2015 Tiancheng "Timothy" Gu
 *
 * Permission is hereby granted, free of charge, to any person obtaining
 * a copy of this software and associated documentation files (the
 * 'Software'), to deal in the Software without restriction, including
 * without limitation the rights to use, copy, modify, merge, publish,
 * distribute, sublicense, and/or sell copies of the Software, and to
 * permit persons to whom the Software is furnished to do so, subject to
 * the following conditions:
 *
 * The above copyright notice and this permission notice shall be
 * included in all copies or substantial portions of the Software.
 *
 * THE SOFTWARE IS PROVIDED 'AS IS', WITHOUT WARRANTY OF ANY KIND,
 * EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF
 * MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT.
 * IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY
 * CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION OF CONTRACT,
 * TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN CONNECTION WITH THE
 * SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.
 */

const charCodeMap = {
    34: '&quot;', // "
    38: '&amp;', // &
    39: '&#39;', // '
    60: '&lt;', // <
    62: '&gt;' // >
}

function escapeHtml(str) {
    let html = ''
    let lastIndex = 0

    for (let i = 0; i < str.length; i++) {
        const escape = charCodeMap[str.charCodeAt(i)]
        if (!escape) continue

        if (lastIndex !== i) {
            html += str.substring(lastIndex, i)
        }

        lastIndex = i + 1
        html += escape
    }

    return html + str.substring(lastIndex)
}

export default escapeHtml;
