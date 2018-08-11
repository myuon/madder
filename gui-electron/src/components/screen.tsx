import * as React from 'react';
import { Communicator, hold } from '../lib';
import { withStyles, StyleRulesCallback } from '@material-ui/core';

const styles: StyleRulesCallback<"root"> = theme => ({
  root: {
  },
});

export class Screen extends React.Component<{com: Communicator}> {
  private screen: React.RefObject<HTMLCanvasElement>;
  private src: string;

  constructor(props: any) {
    super(props);

    this.screen = React.createRef();
    this.src = "";
  }

  public renderScreen(value: number) {
    this.props.com.send(`{
      "method": "Get",
      "path": "/screen/${value}",
      "entity": ${value}
    }`, hold((res: any) => {
      this.src = JSON.parse(res);

      const context = this.screen.current.getContext('2d');
      const image = new Image();
      image.onload = () => {
        context.drawImage(image, 0, 0, 640, 480);
      };
      image.src = JSON.parse(res);
    }));
  }

  render() {
    return (
      <div>
        <canvas ref={this.screen} width="640px" height="480px"></canvas>
      </div>
    );
  }
}

export default withStyles(styles)(Screen);
