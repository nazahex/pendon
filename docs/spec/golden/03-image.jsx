export default function PendonView() { return (<>
<p>An image is <strong>one head</strong> (§7.1): <code>!</code> is the tag, the bracket is the preset&#39;s
<code>alt</code>, the paren its <code>src</code>. A head alone on its line is a block:
</p>
<img alt="Alt text" class="thumb" data-foo="bar" src="https://res.cloudinary.com/x/upload/img.webp" type="imageX" />
<p>Every other position the head carries stays a bare field, so nothing is dropped
(D-7): the quote and the tick keep their slot-qualified path.
</p>
<img alt="Alt text" {...{ "bracket.quote": "bracket quote" }} {...{ "bracket.tick": "bracket tick" }} {...{ "paren.quote": "paren quote" }} {...{ "paren.tick": "paren tick" }} src="https://x.test/src.webp" />
<p>A head that shares its line with text stays inline, and the text after it is
paragraph content, not a caption (§14 retired <code>&lt;figure&gt;</code>):
</p>
<p>Ad ex tempor <img alt="Alt text" class="foo" data-con="jux" src="https://x.test/lazy.webp" type="imageX" /> consectetur.
</p>
<p>The retired marker run is literal text: <code>?</code>, <code>~</code>, <code>w&lt;n&gt;</code>, <code>h&lt;n&gt;</code>, <code>!!</code>, <code>p!</code> and
<code>d!</code> are gone (§14), so only the <code>!</code> that touches <code>[</code> claims anything — here the
run stays as written and the empty-alt head behind it is still an image.
</p>
<p>~?!<img alt="" src="https://x.test/old.webp" />
</p>
<p><code>alt</code> and <code>src</code> are built-in (D-12 rule 5), so an extras item that spells one is
an error — and the head still wins it.
</p>
<img alt="Alt text" class="wide" src="https://x.test/own.webp" />

</>); }
