
use rs_merkle::MerkleTree;
use rs_merkle::algorithms::Sha256;
use rs_merkle::Hasher;


//Dans toutes ces fonctions, initialement les polynômes sont représentés par des vecteurs
//dont les composantes sont les coefficients de ces polynômes dans l'ordre de degré croissant


//Stucture Polynomial avec des méthodes pour travailler sur des polynômes
struct Polynomial {
    coefficients: Vec<i128>,
}

impl Polynomial {

//Détermination d'un polynôme à partir de ses coefficients
     fn new(coefficients: Vec<i128>) -> Self {
         Polynomial { coefficients }
        }

//Evaluation d'un polynôme en un point
     fn evaluate(&self, x: i128) -> i128 {
          let mut result = 0;
          for (i, &coeff) in self.coefficients.iter().enumerate() {
           result += coeff * x.pow(i as u32);
                         }
           result
            }
   
//Addition de deux polynômes
     fn add(&self, other: &Polynomial) -> Polynomial {
          let max_len = self.coefficients.len().max(other.coefficients.len());
          let mut result_coeffs = vec![0;max_len];
          for i in 0..max_len {
          if i < self.coefficients.len() {
             result_coeffs[i] += self.coefficients[i];
               }
          if i < other.coefficients.len() {
                 result_coeffs[i] += other.coefficients[i];
               }
         }
          Polynomial::new(result_coeffs)
      }

 }




//Fonction qui prend en paramètre un domaine FRI, et renvoie le domaine FRI suivant (qui est le domaine dont les éléments sont 
//les carrés des éléments de la première moitié du domaine FRI en paramètre de la fonction)
fn next_fri_domain (domain: Vec<i128>) -> Vec<i128> {
        let mut next_domain = Vec::new();
        for i in 0..domain.len()/2 {
        next_domain.push(domain[i] * domain[i]);
     };
        next_domain
}




//Fonction qui prend en paramètre un polynôme FRI, et renvoie le polynôme FRI suivant en utilisant la strategie séparer
//les parties paires et impaires du polynôme
//Le facteur utilisé pour le calcul du polynôme FRI suivant est constant et fixé à 2
fn next_fri_polynomial (poly: Vec<i128>) -> Vec<i128> {

//on détermine le vecteur even_coeff des coefficients de poly aux indices pairs, et le vecteur odd_coeff
//des coefficients de poly aux indices impairs                
        let mut odd_coeff = Vec::new();
        let mut even_coeff = Vec::new();
        let mut new_coeff = Vec::new();
        for i in 0..poly.len() {
        if i%2 == 0 { even_coeff.push(poly[i]); }
         else { odd_coeff.push(poly[i]); }
      };

//on multiplie la partie aux indices impairs par notre facteur 2 et on détermine le polynôme FRI
        for i in 0..odd_coeff.len() { new_coeff.push(2 * odd_coeff[i]); };
        let p = Polynomial::new(even_coeff);
        let q = Polynomial::new(new_coeff);
        let next_poly = p.add(&q);
        next_poly.coefficients
}





//Fonction qui prend en paramètres un polynôme FRI et un domaine FRI, et qui renvoie un tuple
//composé du domaine FRI suivant, du polynôme FRI suivant, des évaluations FRI suivantes et de la racine
//de Merkle FRI suivante
fn next_fri_layer (poly: Vec<i128>, domain: Vec<i128>) -> ( Vec<i128>, Vec<i128>, Vec<i128>, [u8; 32]) {

//on utilise les fonctions précédentes next_fri_domain et next_fri_polynomial
        let next_domain = next_fri_domain(domain);
        let next_poly = Polynomial::new(next_fri_polynomial(poly));

//Vecteur next_layer des évaluations de next_poly sur next_domain
        let mut next_layer = Vec::new();
        for i in 0..next_domain.len() { next_layer.push(next_poly.evaluate(next_domain[i])); };

//Calcul de la racine de Merkle merkle_root
        let string_values: Vec<String> = next_layer.iter().map(|&x| x.to_string()).collect();
        let hashed_values: Vec<[u8; 32]> = string_values.iter().map(|x| Sha256::hash(x.as_bytes())).collect();
        let merkle_tree = MerkleTree::<Sha256>::from_leaves(&hashed_values);
        let merkle_root = merkle_tree.root().unwrap();
        (next_domain,next_poly.coefficients,next_layer,merkle_root)
}





//Fonction FRI qui prend en paramètres un domaine FRI, un polynôme FRI, le vecteur des évaluations du polynôme sur ce domaine
//et la racine de Merkle correspondant à l'arbre de Merkle de ce vecteur d'évaluations, et qui renvoie le vecteur 
//de tous les domaines FRI, celui de tous les polynômes FRI, celui de toutes les évaluations FRI, et celui de toutes
//les racines de Merkle FRI 
fn fri(domain: Vec<i128>, cp: Vec<i128>, cp_eval: Vec<i128>, cp_r: [u8; 32]) -> (Vec<Vec<i128>>,Vec<Vec<i128>>,Vec<Vec<i128>>,Vec<[u8; 32]>)    
                                                                                                         {
        let mut fri_domain = Vec::new();
        let mut fri_poly = Vec::new();
        let mut fri_layer = Vec::new();
        let mut fri_merkle_root = Vec::new();
        fri_domain.push(domain);
        fri_poly.push(cp);
        fri_layer.push(cp_eval);
        fri_merkle_root.push(cp_r);

//On utilise la fonction précédente next_fri_layer
        while fri_poly[fri_poly.len() - 1].len() > 1 {
let (next_d,next_p,next_l,next_r) = next_fri_layer(fri_poly[fri_poly.len()-1].clone(),fri_domain[fri_domain.len()-1].clone());
        fri_domain.push(next_d);
        fri_poly.push(next_p);
        fri_layer.push(next_l);
        fri_merkle_root.push(next_r);
      };
        (fri_domain,fri_poly,fri_layer,fri_merkle_root)
}




//Test
fn main() {
        // domaine initial
        let domain = vec![1,4,8,16,32,64,128,256];

        // polynôme initial
        let cp = vec![3,1,2,7,3,5];

        // vecteur initial cp_eval des évaluations (qui est le vecteur des évaluations de cp sur domain)
        let cp1 = Polynomial::new(cp.clone());
        let mut cp_eval = Vec::new();
        for i in 0..domain.len() { cp_eval.push(cp1.evaluate(domain[i])); }
        
        // racine de Merkle initiale cp_root (qui est la racine de Merkle de l'arbre de Merkle de cp_eval)
         let string_values: Vec<String> = cp_eval.iter().map(|&x| x.to_string()).collect(); 
         let hashed_values: Vec<[u8; 32]> = string_values.iter().map(|x| Sha256::hash(x.as_bytes())).collect();
         let merkle_tree = MerkleTree::<Sha256>::from_leaves(&hashed_values);
         let cp_root = merkle_tree.root().unwrap(); 
  
        println! ("Polynôme FRI initial : {:?}", cp);
        println! ("Domaine FRI initial : {:?}", domain);
        println! ("Evaluations FRI initiales : {:?}", cp_eval);
        println! ("Racine de Merkle FRI initiale : {:?}", cp_root);
       
        //On utilise l'algorithme fri pour déterminer tous les domaines FRI, polynômes FRI, évaluations FRI, racines de Merkle FRI
        //On trouve le polynôme constant cherché (le dernier polynôme) et ce sont toutes les racines de Merkle qui sont envoyées 
        //au vérificateur
        let tuple = fri(domain,cp,cp_eval,cp_root);
        println! ("Vecteurs des domaines FRI : {:?}", tuple.0);
        println! ("Vecteurs des polynômes FRI : {:?}", tuple.1);
        println! ("Vecteurs des évaluations FRI : {:?}", tuple.2);
        println! ("Vecteurs des racines de Merkle FRI : {:?}", tuple.3);
}
