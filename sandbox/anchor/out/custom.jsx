import Anchor from "@comp/shared/Anchor";

export default function PendonView() { return (<>
<h1>Anchor Custom Node</h1>
<p>External <Anchor attrs={attrs} qux="rox">documentation</Anchor> and internal <Anchor attrs={attrs}>guide</Anchor>.
</p>

</>); }
