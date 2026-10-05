import {
  TableBody,
  TableCellAB,
  TableCellDefault,
  TableFoot,
  TableRowAB,
  TableRowDefault,
} from "@comp/content/Table"

export default function PendonView() {
  return (
    <>
      <p>
        Column extras in the alignment row route the column&#39;s <code>&lt;th&gt;</code> and its{" "}
        <code>&lt;td&gt;</code>
        cells, and the alignment row&#39;s end-of-line extras belong to the{" "}
        <code>&lt;tbody&gt;</code> (§8):
      </p>
      <table class="striped" id="layers" type="tableX">
        <caption class="cap" type="captionAB">
          Layer routing
        </caption>
        <thead>
          <TableRowDefault>
            <TableCellAB type={"cellA"} align={"left"} class={"v-top"}>
              A
            </TableCellAB>
            <TableCellDefault align={"left"}>B</TableCellDefault>
            <TableCellDefault align={"left"}>C</TableCellDefault>
          </TableRowDefault>
        </thead>
        <TableBody class={"body"} type={"tbodyX"}>
          <TableRowAB type={"rowB"} class={"info"}>
            <TableCellAB type={"cellB"} align={"left"} class={"v-top lead"}>
              1
            </TableCellAB>
            <TableCellDefault align={"left"}>2</TableCellDefault>
            <TableCellDefault align={"left"}>3</TableCellDefault>
          </TableRowAB>
          <TableRowDefault>
            <TableCellAB type={"cellA"} align={"left"} class={"v-top wide"} colspan={2} rowspan={2}>
              4
            </TableCellAB>
            <TableCellDefault align={"left"}>5</TableCellDefault>
          </TableRowDefault>
          <TableRowDefault>
            <TableCellDefault align={"left"}>6</TableCellDefault>
            <TableCellDefault align={"left"}>7</TableCellDefault>
          </TableRowDefault>
        </TableBody>
        <TableFoot class={"total"} type={"tfootX"}>
          <TableRowDefault class={"row-danger"}>
            <TableCellAB type={"cellA"} align={"left"} class={"v-top"} colspan={2}>
              Total
            </TableCellAB>
            <TableCellDefault align={"left"}>9</TableCellDefault>
          </TableRowDefault>
        </TableFoot>
      </table>
    </>
  )
}
