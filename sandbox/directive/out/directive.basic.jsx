import DirGen from '@comp/DirGen'

export default function PendonView() { return (<>
<p>Lorem Ipsum
</p>
<DirGen type="DirecA"><p>Lorem DirecA <DirGen type="DirA"> ipsum</DirGen>
</p>
<DirGen type="DirecB"><p>Lorem DirecB <DirGen type="DirA"> ipsum <DirGen type="DirC"> dolor</DirGen></DirGen>.
</p>
</DirGen><p>Lorem DirecA 2
</p>
</DirGen>
</>); }
