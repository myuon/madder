<template>
  <div>
    <canvas ref="screen" width="1280px" height="720px" />
  </div>
</template>

<script>
  import { Communicator, Request, Receiver, cast_as, Component } from "@/lib";

  export default {
    name: 'screen',
    props: [ 'fetchScreen', 'position' ],
    methods: {
      changePosition () {
        this.fetchScreen(this.position, b64str => {
          const src = JSON.parse(b64str);

          const context = this.$refs.screen.getContext('2d');
          const image = new Image();
          image.onload = () => {
            context.drawImage(image, 0, 0, 1280, 720);
          };
          image.src = src;
        });
      }
    },
    watch: {
      position (newPosition, oldPosition) {
        this.changePosition();
      }
    },
    mounted () {
      this.changePosition();
    },
  }
</script>
