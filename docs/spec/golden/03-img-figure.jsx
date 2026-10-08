import AdvancedImage from '@comp/content/AdvancedImage'
import Figure from '@comp/content/Figure'

export default function PendonView() { return (<>
<p>A figure: the extras attach to the outermost node, <code>w</code>/<code>h</code> stay on the inner
<code>&lt;img&gt;</code> and the trailing text is the caption (§7.1).
</p>
<Figure class={"wide"} container={"figure"} id={"fig-1"} isLazy={true} style={"--rotate: 5deg;"} type={"figureX"}><AdvancedImage alt={"Alt text"} async_decoding={1} height={300} lazy={1} src={"https://res.cloudinary.com/x/upload/fig.webp"} width={800}></AdvancedImage>Afigurecaptionwith<strong>markup</strong>.</Figure><p>An image node (<code>~?!</code>) uses the <code>img</code> layer alone:
</p>
<AdvancedImage alt={"Alt text"} async_decoding={1} class={"thumb"} foo={"bar"} lazy={1} src={"https://res.cloudinary.com/x/upload/img.webp"} type={"imageX"} width={600}></AdvancedImage><p>A lazy image inline attaches its extras to the <code>&lt;img&gt;</code> element:
</p>
<p>Ad ex tempor <AdvancedImage alt={"Alt text"} async_decoding={1} class={"foo"} con={"jux"} lazy={1} src={"https://res.cloudinary.com/x/upload/lazy.webp"} type={"imageX"} width={300}></AdvancedImage> consectetur.
</p>

</>); }
