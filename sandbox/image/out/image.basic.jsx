import AdvancedImage from '@comp/content/AdvancedImage'
import { FigureX } from '@comp/content/Figure'

export const references = {};
export const citations = [];
export default function PendonView() { return (<>
<p>Esse exercitation irure ullamco veniam fugiat eu consequat proident quis do.
</p>
<p><AdvancedImage alt="lorem ipsum" async_decoding={1} baz={23} foo="bar" height={300} lazy={1} src="https://example/foo.png" style="--wix: sum; --rotate: 5deg;" type="TypeX" width={800}></AdvancedImage> Caption
</p>
<FigureX class="wide" container="figure" id="fig-1" isLazy={true} style="--rotate: 5deg;" type="figureX"><AdvancedImage alt="Alt text" async_decoding={1} height={300} lazy={1} src="https://res.cloudinary.com/x/upload/fig.webp" width={800}></AdvancedImage>A figure caption with <strong>markup</strong>.</FigureX>
</>); }
