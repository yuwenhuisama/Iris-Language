#include "IrisComponents/IrisParts/IrisFieldIdentifier.h"
#include "IrisUnil/IrisIdentifier.h"


IrisFieldIdentifier::IrisFieldIdentifier(IrisList<IrisIdentifier*>* pList, IrisIdentifier* pIdentifier, bool bIsTopField) : m_pList(pList), m_pIdentifier(pIdentifier), m_bIsTopField(bIsTopField)
{
}


IrisFieldIdentifier::~IrisFieldIdentifier()
{
	if (m_pList) {
		m_pList->Ergodic([](IrisIdentifier*& x) ->bool { delete x; x = nullptr; return true; });
		m_pList->Clear();
		delete m_pList;
	}
	if (m_pIdentifier)
		delete m_pIdentifier;
}
