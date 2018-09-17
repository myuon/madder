<template>
  <div v-if="component != null">
    <vs-input disabled vs-label="id" class="inputx" placeholder="Disabled" v-model="component.id" />
    <vs-input disabled vs-label="component_type" class="inputx" placeholder="Disabled" v-model="component.component_type" />
    <vs-input @change="$emit('change-attr', 'start_time', component.start_time)" vs-label="start_time" v-model.number="component.start_time" type="number" />
    <vs-input @change="$emit('change-attr', 'length', component.length)" vs-label="length" v-model.number="component.length" type="number" />
  </div>
</template>

<script>
  export default {
    name: 'component-detail',
    props: [ 'component' ],
  }
</script>
