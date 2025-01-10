export function get_bit_name() {
    const urlParams = new URLSearchParams(window.location.search);
    return urlParams.get('bit');
}