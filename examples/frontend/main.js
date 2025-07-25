const { fromEvent, switchMap } = rxjs;
const result = document.getElementById("result");

fromEvent(document.getElementById("pay"), 'click')
  .pipe(
    switchMap(() =>
      fetch("http://localhost:8081/health").then(res => res.text())
    )
  )
  .subscribe(data => {
    result.innerText = data;
  });
