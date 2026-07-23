// TSX fixture: the JS-family constructs plus JSX and TypeScript types together,
// which is what the tsx grammar exists to parse.

interface Item {
  id: number;
  name: string;
  hidden?: boolean;
}

function Badge(props: { label?: string }): JSX.Element | null {
  if (!props.label) {
    return null;
  }
  return <span className="badge">{props.label}</span>;
}

const List = (items: Item[]): JSX.Element => {
  return (
    <ul>
      {items.map((item) =>
        item.hidden ? null : <li key={item.id}>{item.name}</li>
      )}
    </ul>
  );
};

class Panel<T extends Item> {
  private rows: T[];

  constructor(rows: T[]) {
    this.rows = rows;
  }

  render(): JSX.Element {
    const body = this.rows && this.rows.length ? this.rows : [];
    return (
      <div>
        {body.map((r) => {
          if (r.hidden) {
            return null;
          }
          return <Badge label={r.name} />;
        })}
      </div>
    );
  }
}

function classify(n: number): string {
  switch (true) {
    case n < 0:
      return "negative";
    default:
      return n === 0 ? "zero" : "positive";
  }
}
