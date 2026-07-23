// JSX fixture. `.jsx` routes to the same mozjs parser as `.js`, so this checks
// that JSX syntax does not perturb the native metrics relative to rca.

function Badge(props) {
  if (!props.label) {
    return null;
  }
  return <span className="badge">{props.label}</span>;
}

const List = (items) => {
  return (
    <ul>
      {items.map((item) =>
        item.hidden ? null : <li key={item.id}>{item.name}</li>
      )}
    </ul>
  );
};

class Panel {
  render(rows) {
    const body = rows && rows.length ? rows : [];
    return <div>{body.map((r) => <Badge label={r} />)}</div>;
  }
}
