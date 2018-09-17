<template>
  <div :style="timelineStyle()">
    <template v-for="(component, index) in Array.from(components.values())">
      <div
        class="component"
        :key="component.id"
        :style="componentStyle(component, index)"
        @click='$emit("select", component.id)'
      >
        {{ component.id.slice(0,5) }}
      </div>
    </template>
  </div>
</template>

<script>
  export default {
    name: 'timeline',
    props: [ 'components', 'selected' ],
    methods: {
      componentStyle (component, index) {
        return {
          position: 'absolute',
          top: `${index * 30}px`,
          left: `${component.start_time}px`,
          width: `${component.length}px`,
          display: 'block',
          backgroundColor:
            component.component_type === 'Sound'
              ? component.id === this.selected ? '#ccf' : '#99f'
              : component.id === this.selected ? '#fcc' : '#f99',
          padding: '5px',
        };
      },
      timelineStyle () {
        return {
          position: 'relative',
          height: '10rem',
          overflowY: 'scroll',
        }
      }
    },
  }
</script>
