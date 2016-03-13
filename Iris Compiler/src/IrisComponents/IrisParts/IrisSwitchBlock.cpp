#include "IrisComponents/IrisParts/IrisSwitchBlock.h"
#include "IrisComponents/IrisParts/IrisWhen.h"
#include "IrisUnil/IrisBlock.h"


IrisSwitchBlock::IrisSwitchBlock(IrisList<IrisWhen*>* pWhenList, IrisBlock* pElseBlock) : m_pWhenList(pWhenList), m_pElseBlock(pElseBlock)
{
}


IrisSwitchBlock::~IrisSwitchBlock()
{
	if (m_pWhenList) {
		m_pWhenList->Ergodic([](IrisWhen*& x) -> bool { delete x; x = nullptr; return true; });
		m_pWhenList->Clear();
		delete m_pWhenList;
	}
	if (m_pElseBlock)
		delete m_pElseBlock;
}
