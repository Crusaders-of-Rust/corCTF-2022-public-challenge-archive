function totalPrice(val) {
    qty = parseInt(val)
    total = document.getElementById('total');   
    total.value = "$" + ((isNaN(qty) || qty > 1000) ? "?" : (1337.00 * qty).toFixed(2)) + "/day";
}

function updateRequirements(val) {
    promo_how = document.getElementById('promo_how');
    promo_how.required = val ? true : false;   
}