import Parego from "@comp/shared/Parego"
import Hint from "./Hint"

export default function PendonView() {
  return (
    <>
      <h1>Aside Demo</h1>
      <p>:::tip</p>
      <p>
        Officia laboris ad sunt adipisicing dolor. Adipisicing amet eiusmod enim veniam et
        reprehenderit. Magna est dolor mollit et nisi amet commodo quis amet cillum aute{" "}
        <strong>consectetur</strong> aliquip. Enim ad velit reprehenderit Lorem proident magna.
        Aliqua sunt elit reprehenderit irure voluptate voluptate pariatur dolor.
      </p>
      <blockquote>
        <p>Velit aute culpa proident labore officia.</p>
      </blockquote>
      <Hint type="?">
        <p>In pariatur duis eiusmod id enim cillum proident cupidatat amet Lorem minim.</p>
      </Hint>
      <p>:::</p>
      <p>Ullamco est velit dolore mollit qui ut.</p>
      <Parego type="warn" title="Foo Bar">
        <p>
          Lorem officia <i>consequat</i> labore exercitation eu. Sit consectetur mollit ex consequat
          magna consequat culpa excepteur. Veniam ullamco id ipsum Lorem incididunt id. Ea aute
          tempor adipisicing veniam adipisicing minim eiusmod aute magna velit minim non laboris
          est. Magna aliqua eu ea ad qui in excepteur in. Culpa nostrud labore cillum minim magna
          enim consequat culpa dolor ad. Tempor amet velit occaecat ad adipisicing quis sunt duis
          dolor deserunt incididunt ullamco nostrud fugiat.
        </p>
        <p>Laborum sunt labore est eu eiusmod laboris deserunt consequat incididunt quis qui id.</p>
      </Parego>
      <h2>Duis excepteur dolor amet consectetur</h2>
      <p>
        Sunt irure aute sit minim adipisicing non reprehenderit aliqua. Ut veniam anim aliqua
        eiusmod irure laborum officia nostrud tempor laboris mollit deserunt ut anim. Adipisicing
        ullamco Lorem id nulla qui laborum.
      </p>
      <Parego type="info" title="">
        <p>
          Ea incididunt eu aliquip ea eiusmod. Fugiat labore ut id anim consequat do. Veniam
          consequat adipisicing do tempor sit fugiat. Ullamco sint enim culpa velit. Magna deserunt
          tempor reprehenderit nisi dolore anim sint eiusmod amet qui voluptate eiusmod exercitation
          commodo. Lorem ad eiusmod duis proident eu.
        </p>
      </Parego>
    </>
  )
}
