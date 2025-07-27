const { fromEvent, switchMap } = rxjs;
const result = document.getElementById("result");

fromEvent(document.getElementById("checkHealth"), 'click')
  .pipe(
    switchMap(() =>
      fetch("http://localhost:8081/health").then(res => res.text())
    )
  )
  .subscribe(data => {
    result.innerText = data;
  });



const subscribeBtn = document.getElementById("subscribe");
const status = document.getElementById("status");

fromEvent(subscribeBtn, "click")
  .pipe(
    switchMap(() =>
      fetch("http://localhost:8080/generate-stripe-checkout", {
        method: "POST"
      }).then(res => res.json())
    )
  )
  .subscribe(({ url }) => {
    status.innerText = "Redirecting to Stripe...";
    window.location.href = url;
  });