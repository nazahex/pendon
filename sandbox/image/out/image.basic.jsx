import AdvancedImage from '@comp/content/AdvancedImage'
import Figure from '@comp/content/Figure'

export const references = {};
export const citations = [];
export default function PendonView() { return (<>
<p>Esse exercitation irure ullamco veniam fugiat eu consequat proident quis do.
</p>
<Figure baz={23} container="figure" foo="bar" style="--wix: sum; --rotate: 5deg;"><AdvancedImage alt="lorem ipsum" async_decoding={1} height={300} lazy={1} src="https://example/foo.png" width={800}></AdvancedImage>Caption</Figure>
</>); }
