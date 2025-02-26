import { Cell, Column, Row, Table } from "@tanstack/react-table";
import { Checkbox } from "../ui/checkbox";
import { useState } from "react";


// @ts-expect-error meta function
export default function EditableCheckBox({ row, column, table, getValue }) {
  const initialValue = getValue() as boolean;
  const [value, setValue] = useState(initialValue);
  return (
    <Checkbox
      checked={value}
      onCheckedChange={(newValue: boolean) => {
        table.options.meta?.updateData(row.index, column.columnDef.accessorKey, newValue, row.getValue("commandName"));
        setValue(newValue);
      }}
      aria-label="Enable trigger."
    />
  );
}
