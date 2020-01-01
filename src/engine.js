function View(props, children = []) {
  this.props = props;
  this.children = children;
  this.node = createNode(
    children.map(child => child.node),
    props.style || {},
  );
}

View.prototype.render = function () {
  const {node, children, props} = this;

  const {top, left, width, height} = getLayout(node);
  const {background = '#ffffff'} = props.style;

  let html = `<div style="position:absolute;background:${background};top:${top}px;left:${left}px;width:${width}px;height:${height}px;">`;
  const childHtml = children.map(child => child.render()).join('');
  return html + childHtml + '</div>';
};

// factory to create jsx instance
function createElement(Constructor, props, ...children) {
  print(JSON.stringify(props));
  return new Constructor(props, children);
}
