import { HeadingDefault, HeadingXY } from "@comp/content/Heading"

export default function PendonView() {
  return (
    <>
      <p>
        <code>[slug]</code>, <code>(&quot;title&quot;)</code> and <code>@@type&#123;…&#125;</code>{" "}
        are all optional and independent (§7.4).
      </p>
      <HeadingXY
        level={"3"}
        type={"headingX"}
        number={"1."}
        class={"fancy"}
        id={"slug-head"}
        raw_title={"Extras-routed heading"}
        title={"Heading X"}>
        Extras-routed heading
      </HeadingXY>
      <HeadingXY
        level={"3"}
        type={"headingY"}
        number={"auto"}
        id={"heading-y"}
        raw_title={"Routed by the second marker"}
        title={"Heading Y"}>
        Routed by the second marker
      </HeadingXY>
      <HeadingDefault
        class={"x"}
        id={"unrouted"}
        level={3}
        number={3}
        raw_title={"An unclaimed marker keeps the layer default"}
        type={"unknownZ"}>
        An unclaimed marker keeps the layer default
      </HeadingDefault>
      <HeadingDefault
        level={2}
        number={1}
        raw_title={"A plain heading without extras"}
        slug={"a-plain-heading-without-extras"}>
        A plain heading without extras
      </HeadingDefault>
    </>
  )
}
