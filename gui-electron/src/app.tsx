import * as React from "react";
import * as ReactDOM from "react-dom";
import Button from '@material-ui/core/Button';

class App extends React.Component {
  handleClick = () => {
    alert('Clicked!');
  }

  render() {
    return (
      <div>
        <Button onClick={this.handleClick}>Push meeeeeeeeeeeeeee!</Button>
      </div>
    );
  }
}

ReactDOM.render(<App />, document.getElementById("app"));
