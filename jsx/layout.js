function render() {
  return (
    <View style={{
      width: 400,
      height: 400,
      flexDirection: 'column'
    }}>
      <View style={{
        flexGrow: 0.5,
        background: 'green'
      }}>
        <View style={{
          flexGrow: 0.5,
          background: '#81D4FA'
        }}/>
        <View style={{
          flexGrow: 0.5,
          flexDirection: 'column'
        }}>
          <View style={{
            flexGrow: 0.5,
            background: '#80CBC4'
          }}/>
          <View style={{
            flexGrow: 0.5,
            background: '#3D5AFE'
          }}/>
        </View>
      </View>
      <View style={{
        flexGrow: 0.5,
        flexDirection: 'row',
        flexWrap: 'wrap'
      }}>
        <View style={{
          flexGrow: 0.5,
          background: '#6200EE'
        }}/>
        <View style={{
          flexGrow: 0.3,
          background: '#448AFF'
        }}/>
        <View style={{
          flexGrow: 1,
          width: 300,
          background: '#F3D8BD'
        }}/>
      </View>
    </View>
  );
}
